pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Eclipse's Choose a console: the makers in a column, the chosen maker's consoles as plates laid out by the kit's GroupGridLayout, the focused console's disc, name and facts beside them (EmuSen_Qt.md §3.4.9).
ThemeScreen {
    id: screen
    objectName: "consolesScreen"
    role: ThemeScreen.Consoles
    focusItems: [
        root.firstPlate
    ]
    NormalizedCanvas {
        id: canvas
        objectName: "consolesCanvas"
        FocusScope {
            id: root
            objectName: "consoles"
            anchors.fill: parent
            focus: true
            // A board pixel in canvas units (the table is eclipse_consoles.tsv).
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
            // The chosen maker's group, the console the aside shows, and the plate last focused.
            property int chosenGroup: 0
            property Item shown: null
            property Item lastPlate: null
            // The plate the screen opens on: the console last opened, else the first maker's first console; and the maker focused.
            property Item firstPlate: null
            property Item focusedMaker: null
            // How many delegates are made; a neighbour is looked up again as each is.
            property int made: 0
            Board {
                id: board
                designHeight: canvas.designHeight
            }
            Ink {
                id: ink
            }
            GroupGridLayout {
                id: layout
                model: Library.consoles
                columns: root.columns
            }
            Connections {
                target: Library.consoles
                function onModelReset() { layout.read() }
                function onDataChanged() { layout.read() }
            }
            // The two colour fields behind (board lines 19 and 20): constants of the design, on the screen's ground.
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
                        color: "#5B3FD6"
                        blur: 140 * root.px
                        opacity: 0.5
                    },
                    BackdropField {
                        x: 1500 * root.px
                        y: 520 * root.px
                        width: 620 * root.px
                        height: 620 * root.px
                        color: "#C0486F"
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

                    // The header: the mark and the program's name, and the status at the right (board lines 22 to 40); the sections' place is left empty (R-20261115-7).
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
                        x: root.width - root.margin - width
                        y: 80 * root.px - height / 2
                    }

                    // The makers, one row each at its group's place, the chosen one filled (board lines 54 to 60).
                    Flickable {
                        id: makerList
                        objectName: "makers"
                        x: root.margin
                        y: root.listTop
                        width: root.makerWidth
                        height: root.foot - root.listTop
                        clip: true
                        interactive: false
                        contentHeight: layout.groups * root.makerPitch
                        contentY: Math.max(0, Math.min(root.chosenGroup * root.makerPitch + root.makerHeight - height, contentHeight - height))
                        Repeater {
                            id: makers
                            model: Library.consoles
                            onItemAdded: root.made = root.made + 1
                            onItemRemoved: root.made = root.made - 1
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
                                KeyNavigation.up: root.made > 0 && maker.before >= 0 ? makers.itemAt(maker.before) : null
                                KeyNavigation.down: root.made > 0 && maker.after >= 0 ? makers.itemAt(maker.after) : null
                                KeyNavigation.right: root.made > 0 && maker.first >= 0 ? plates.itemAt(maker.first) : null
                                onActiveFocusChanged: {
                                    root.focusedMaker = maker.activeFocus ? maker : (root.focusedMaker === maker ? null : root.focusedMaker)
                                    if (maker.activeFocus)
                                        root.chosenGroup = maker.group
                                }
                                onActivated: plates.itemAt(maker.first).forceActiveFocus()
                                Rectangle {
                                    objectName: "makerFill." + maker.group
                                    anchors.fill: parent
                                    radius: 14 * root.px
                                    color: maker.chosen ? Tokens.accent : "transparent"
                                }
                                Text {
                                    objectName: "makerName." + maker.group
                                    x: 18 * root.px
                                    width: parent.width - 2 * x - makerCount.implicitWidth - 12 * root.px
                                    height: parent.height
                                    verticalAlignment: Text.AlignVCenter
                                    elide: Text.ElideRight
                                    text: maker.maker
                                    color: maker.chosen ? Tokens.overAccent : ink.help
                                    font.pixelSize: Type.size(22 * root.px, Tokens.textScale)
                                    font.weight: maker.chosen ? Font.DemiBold : Font.Normal
                                }
                                Text {
                                    id: makerCount
                                    objectName: "makerCount." + maker.group
                                    x: parent.width - 18 * root.px - width
                                    height: parent.height
                                    verticalAlignment: Text.AlignVCenter
                                    text: maker.groupSize
                                    color: maker.chosen ? Tokens.overAccent : ink.help
                                    font.pixelSize: Type.size(18 * root.px, Tokens.textScale)
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
                        height: root.foot - root.listTop
                        leftMargin: root.plateGap / 2
                        rightMargin: root.plateGap / 2
                        clip: true
                        interactive: false
                        readonly property real pitch: root.plateHeight + root.plateGap
                        contentHeight: Math.max(0, (layout.rows[root.chosenGroup] ?? 0) * grid.pitch - root.plateGap)
                        contentY: root.lastPlate ? Math.max(0, Math.min(root.lastPlate.y + root.plateHeight - height, contentHeight - height)) : 0
                        Repeater {
                            id: plates
                            model: Library.consoles
                            onItemAdded: (index, item) => {
                                root.made = root.made + 1
                                root.firstPlate = item.opening ? item : root.firstPlate
                            }
                            onItemRemoved: root.made = root.made - 1
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
                                required property int group
                                required property int groupIndex
                                required property int groupSize
                                readonly property int leftIndex: layout.left[plate.index] ?? -1
                                readonly property int rightIndex: layout.right[plate.index] ?? -1
                                readonly property int upIndex: layout.up[plate.index] ?? -1
                                readonly property int downIndex: layout.down[plate.index] ?? -1
                                readonly property int previousFirst: layout.firsts[plate.group - 1] ?? -1
                                readonly property int nextFirst: layout.firsts[plate.group + 1] ?? -1
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
                                // Inside the grid by the layout's walk; past its left edge the maker, past its right Open.
                                KeyNavigation.left: root.made > 0 ? (plate.leftIndex >= 0 ? plates.itemAt(plate.leftIndex) : makers.itemAt(layout.firsts[plate.group] ?? 0)) : null
                                KeyNavigation.right: root.made > 0 && plate.rightIndex >= 0 ? plates.itemAt(plate.rightIndex) : openButton
                                KeyNavigation.up: root.made > 0 && plate.upIndex >= 0 ? plates.itemAt(plate.upIndex) : null
                                KeyNavigation.down: root.made > 0 && plate.downIndex >= 0 ? plates.itemAt(plate.downIndex) : null
                                // LB and RB: the previous and next maker's first console.
                                Keys.onPressed: (event) => {
                                    const to = event.key === Qt.Key_PageUp ? plate.previousFirst : (event.key === Qt.Key_PageDown ? plate.nextFirst : -1)
                                    event.accepted = to >= 0
                                    if (to >= 0)
                                        plates.itemAt(to).forceActiveFocus()
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
                                    y: parent.height - 20 * root.px - height
                                    spacing: 6 * root.px
                                    Text {
                                        objectName: "plateName." + plate.index
                                        width: parent.width
                                        text: plate.displayName
                                        wrapMode: Text.WordWrap
                                        maximumLineCount: 2
                                        elide: Text.ElideRight
                                        color: Tokens.text
                                        font.pixelSize: Type.size(21 * root.px, Tokens.textScale)
                                        font.weight: Font.DemiBold
                                        lineHeight: 1.15
                                    }
                                    Text {
                                        objectName: "plateLine." + plate.index
                                        width: parent.width
                                        elide: Text.ElideRight
                                        text: Words.line([plate.year > 0 ? String(plate.year) : "", Words.games(plate.count)])
                                        color: Tokens.muted
                                        font.pixelSize: Type.size(16 * root.px, Tokens.textScale)
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
                    }

                    // The focused console: its disc, its maker and year, its name, its facts with its core's mark, and Open (board lines 77 to 97).
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
                            // The focused console's own colour for the rim and both glows (R-20261115-9).
                            corona: root.shown ? root.shown.glow : "transparent"
                        }
                        Column {
                            y: disc.height + (22 + 12) * root.px
                            width: parent.width
                            spacing: 8 * root.px
                            Text {
                                id: eyebrow
                                objectName: "consoleEyebrow"
                                width: parent.width
                                elide: Text.ElideRight
                                text: root.shown ? Words.line([root.shown.maker, root.shown.year > 0 ? String(root.shown.year) : ""]) : ""
                                color: ink.rose
                                font.pixelSize: Type.size(18 * root.px, Tokens.textScale)
                                font.weight: Font.DemiBold
                                // The board's 0.2 em (line 80).
                                font.letterSpacing: 0.2 * eyebrow.font.pixelSize
                                font.capitalization: Font.AllUppercase
                            }
                            Text {
                                objectName: "consoleName"
                                width: parent.width
                                wrapMode: Text.WordWrap
                                maximumLineCount: 2
                                elide: Text.ElideRight
                                text: root.shown ? root.shown.displayName : ""
                                color: Tokens.text
                                font.pixelSize: Type.size(46 * root.px, Tokens.textScale)
                                font.weight: Font.DemiBold
                                lineHeight: 1.08
                            }
                            Item {
                                width: parent.width
                                height: 14 * root.px
                            }
                            Item {
                                width: parent.width
                                height: facts.height
                                PlanetMark {
                                    objectName: "consoleCoreMark"
                                    visible: root.shown !== null && root.shown.engineMark.length > 0
                                    // Centred on the facts' first line.
                                    y: (factsLine.height - height) / 2
                                    width: 24 * root.px
                                    height: 24 * root.px
                                    mark: root.shown ? root.shown.engineMark : ""
                                }
                                FontMetrics {
                                    id: factsLine
                                    font: facts.font
                                }
                                Text {
                                    id: facts
                                    objectName: "consoleFacts"
                                    x: 34 * root.px
                                    width: parent.width - x
                                    wrapMode: Text.WordWrap
                                    // The facts with the played count after the games' count (R-20261115-8).
                                    text: root.shown ? Words.line([Words.facts(root.shown.maker, root.shown.year, root.shown.count, ""), Words.played(root.shown.played), Words.runsOn(root.shown.engineName, root.shown.engineKind)]) : ""
                                    color: Tokens.muted
                                    font.pixelSize: Type.size(21 * root.px, Tokens.textScale)
                                }
                            }
                            Item {
                                width: parent.width
                                height: 8 * root.px
                            }
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
                                // This board's Open has no halo and no glow (board line 90).
                                glows: false
                                labelSize: 23
                                lead: 12
                                trail: 28
                                gap: 12
                                KeyNavigation.left: root.lastPlate
                                onActivated: Actions.openConsole(root.shown?.key ?? "")
                            }
                            // The A entry's words, read from the help bar's own list.
                            Repeater {
                                model: Help.entries(Help.Console)
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
                                }
                            }
                            QtObject {
                                id: acceptWords
                                property string text
                            }
                        }
                    }

                    // No console holds a game: the program's empty words.
                    Column {
                        objectName: "noConsoles"
                        visible: Library.consoles.count === 0
                        y: 537 * root.px
                        width: root.width
                        spacing: 12 * root.px
                        Text {
                            width: parent.width
                            horizontalAlignment: Text.AlignHCenter
                            text: Words.noConsoles
                            color: Tokens.muted
                            font.pixelSize: Type.size(22 * root.px, Tokens.textScale)
                        }
                        Text {
                            width: parent.width
                            horizontalAlignment: Text.AlignHCenter
                            text: Words.addFolder
                            color: Tokens.muted
                            font.pixelSize: Type.size(22 * root.px, Tokens.textScale)
                        }
                    }
                }
            }
            // The help line at the foot, and at its right the chosen maker and where the focused console stands in it (board lines 99 to 105).
            HelpLine {
                objectName: "consolesHelp"
                unit: root.px
                controller: frame.controller
                model: Help.entries(Help.Console, {group: true})
                ink: ink.help
                // This board's foot: its 34 px row 40 px over the bottom (board line 99).
                middle: 1143
                Text {
                    objectName: "consolePlace"
                    visible: root.shown !== null
                    text: root.shown ? Words.line([root.shown.maker, Words.place(root.shown.groupIndex, root.shown.groupSize)]) : ""
                    color: ink.help
                    font.pixelSize: Type.size(21 * root.px, Tokens.textScale)
                }
            }
        }
    }
}
