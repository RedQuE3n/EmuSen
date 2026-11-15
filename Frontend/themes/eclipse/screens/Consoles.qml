pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme

// Eclipse's Choose a console: the makers in a column, the chosen maker's consoles as plates, the focused console's disc, name and facts beside them (EmuSen_Qt.md §3.4.8).
ThemeScreen {
    id: screen
    objectName: "consolesScreen"
    role: ThemeScreen.Consoles
    NormalizedCanvas {
        id: canvas
        objectName: "consolesCanvas"
        FocusScope {
            id: root
            objectName: "consoles"
            anchors.fill: parent
            focus: true
            // The board's pixel in design units: the canvas's height over the board's 1200 px, a constant of the design (the table is eclipse_consoles.tsv).
            readonly property real px: canvas.designHeight / 1200
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
            // Board colours Home's board draws too, to be read from Home's parts/Ink.qml: the eyebrow's rose (line 80), the makers' ink (line 56).
            readonly property color eyebrowInk: "#F3C9D2"
            readonly property color makerInk: "#D8DBE6"
            readonly property real asideX: root.width - root.margin - root.asideWidth
            readonly property real gridWidth: root.asideX - root.asideGap - root.gridX
            readonly property int columns: Math.max(1, Math.floor((root.gridWidth + root.plateGap) / (root.plateWidth + root.plateGap)))
            // The chosen maker's group, and the console the aside shows: the focused plate, else the chosen group's first.
            property int chosenGroup: 0
            property var shown: null
            property var lastPlate: null

            function plateAt(group, index) {
                for (let i = 0; i < plates.count; i++) {
                    const p = plates.itemAt(i)
                    if (p && p.group === group && p.groupIndex === index)
                        return p
                }
                return null
            }
            function makerOf(group) {
                for (let i = 0; i < makers.count; i++) {
                    const m = makers.itemAt(i)
                    if (m && m.group === group && m.groupIndex === 0)
                        return m
                }
                return null
            }
            function groupCount() {
                let n = 0
                for (let i = 0; i < plates.count; i++) {
                    const p = plates.itemAt(i)
                    if (p)
                        n = Math.max(n, p.group + 1)
                }
                return n
            }
            // The share of a console's games played, at least the board's sliver (0.04) for one with games.
            function share(played, count) {
                return count > 0 ? Math.max(played / count, 0.04) : 0
            }
            function reach(item) {
                if (item)
                    item.forceActiveFocus()
            }
            function stepGroup(by) {
                const n = root.groupCount()
                if (n > 0)
                    root.reach(root.plateAt(((root.chosenGroup + by) % n + n) % n, 0))
            }

            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: true
                padFamily: Device.family
                focus: true
                controller.navigationShown: false
                help: Help.entries(Help.Console, {group: true})
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
                        contentHeight: root.groupCount() * root.makerPitch
                        contentY: Math.max(0, Math.min(root.chosenGroup * root.makerPitch + root.makerHeight - height, contentHeight - height))
                        Repeater {
                            id: makers
                            model: Library.consoles
                            Walkable {
                                id: maker
                                required property int index
                                required property string maker
                                required property int group
                                required property int groupIndex
                                required property int groupSize
                                readonly property bool chosen: maker.group === root.chosenGroup
                                controller: frame.controller
                                objectName: "maker." + maker.group
                                visible: maker.groupIndex === 0
                                Accessible.name: maker.maker
                                Accessible.description: Words.games(maker.groupSize)
                                y: maker.group * root.makerPitch
                                width: root.makerWidth
                                height: root.makerHeight
                                onActiveFocusChanged: {
                                    if (!maker.activeFocus)
                                        return
                                    root.chosenGroup = maker.group
                                    root.shown = root.plateAt(maker.group, 0)
                                }
                                onActivated: root.reach(root.plateAt(maker.group, 0))
                                Keys.onRightPressed: root.reach(root.plateAt(maker.group, 0))
                                Keys.onUpPressed: root.reach(root.makerOf(maker.group - 1))
                                Keys.onDownPressed: root.reach(root.makerOf(maker.group + 1))
                                Rectangle {
                                    objectName: "makerFill." + maker.group
                                    anchors.fill: parent
                                    radius: 14 * root.px
                                    color: maker.chosen ? Tokens.accent : "transparent"
                                }
                                FocusRing {
                                    objectName: "makerRing." + maker.group
                                    anchors.fill: parent
                                    radius: 14 * root.px
                                    visible: maker.activeFocus
                                }
                                Text {
                                    objectName: "makerName." + maker.group
                                    x: 18 * root.px
                                    width: parent.width - 24 - makerCount.implicitWidth - 8
                                    height: parent.height
                                    verticalAlignment: Text.AlignVCenter
                                    elide: Text.ElideRight
                                    text: maker.maker
                                    color: maker.chosen ? Tokens.overAccent : root.makerInk
                                    font.pixelSize: 22 * root.px
                                    font.weight: maker.chosen ? Font.DemiBold : Font.Normal
                                }
                                Text {
                                    id: makerCount
                                    objectName: "makerCount." + maker.group
                                    x: parent.width - 12 - width
                                    height: parent.height
                                    verticalAlignment: Text.AlignVCenter
                                    text: maker.groupSize
                                    color: maker.chosen ? Tokens.overAccent : root.makerInk
                                    font.pixelSize: 18 * root.px
                                }
                            }
                        }
                    }

                    // The chosen maker's consoles as plates, placed by their place in the group (board lines 62 to 75).
                    Flickable {
                        id: grid
                        objectName: "plates"
                        x: root.gridX - root.plateGap / 2
                        y: root.listTop
                        width: root.gridWidth + root.plateGap
                        height: root.foot - root.listTop
                        clip: true
                        interactive: false
                        // Half a gap past the grid on each side, so the clip never shaves an outer plate's edge.
                        leftMargin: root.plateGap / 2
                        rightMargin: root.plateGap / 2
                        readonly property int rows: root.lastPlate ? Math.ceil(root.lastPlate.groupSize / root.columns) : 0
                        readonly property real pitch: root.plateHeight + root.plateGap
                        contentHeight: Math.max(0, grid.rows * grid.pitch - root.plateGap)
                        contentY: root.lastPlate ? Math.max(0, Math.min(root.lastPlate.y + root.plateHeight - height, contentHeight - height)) : 0
                        Repeater {
                            id: plates
                            model: Library.consoles
                            Walkable {
                                id: plate
                                required property int index
                                required property string key
                                required property string displayName
                                required property string maker
                                required property int year
                                required property int count
                                required property real hue
                                required property string engineName
                                required property string engineKind
                                required property string engineMark
                                required property bool libretro
                                required property int group
                                required property int groupIndex
                                required property int groupSize
                                required property int played
                                required property string glow
                                readonly property int column: plate.groupIndex % root.columns
                                controller: frame.controller
                                objectName: "plate." + plate.index
                                visible: plate.group === root.chosenGroup
                                // The console last opened has the focus, else the first maker's first console.
                                focus: Browse.section.length > 0 ? plate.key === Browse.section : plate.group === 0 && plate.groupIndex === 0
                                Accessible.name: plate.displayName
                                Accessible.description: Words.facts(plate.maker, plate.year, plate.count, Words.runsOn(plate.engineName, plate.engineKind))
                                x: plate.column * (root.plateWidth + root.plateGap)
                                y: Math.floor(plate.groupIndex / root.columns) * grid.pitch
                                width: root.plateWidth
                                height: root.plateHeight
                                onActiveFocusChanged: {
                                    if (!plate.activeFocus)
                                        return
                                    root.chosenGroup = plate.group
                                    root.shown = plate
                                    root.lastPlate = plate
                                    Actions.focus(plate.key)
                                }
                                onActivated: Actions.openConsole(plate.key)
                                Keys.onLeftPressed: root.reach(plate.column > 0 ? root.plateAt(plate.group, plate.groupIndex - 1) : root.makerOf(plate.group))
                                Keys.onRightPressed: root.reach(plate.column < root.columns - 1 && plate.groupIndex + 1 < plate.groupSize ? root.plateAt(plate.group, plate.groupIndex + 1) : openButton)
                                Keys.onUpPressed: root.reach(root.plateAt(plate.group, plate.groupIndex - root.columns))
                                Keys.onDownPressed: {
                                    const below = Math.min(plate.groupIndex + root.columns, plate.groupSize - 1)
                                    if (Math.floor(below / root.columns) > Math.floor(plate.groupIndex / root.columns))
                                        root.reach(root.plateAt(plate.group, below))
                                }
                                Keys.onPressed: event => {
                                    if (event.key === Qt.Key_PageUp || event.key === Qt.Key_PageDown) {
                                        root.stepGroup(event.key === Qt.Key_PageUp ? -1 : 1)
                                        event.accepted = true
                                    }
                                }
                                Rectangle {
                                    objectName: "plateFill." + plate.index
                                    anchors.fill: parent
                                    radius: 18 * root.px
                                    color: Tokens.inputSurface
                                    border.width: plate.activeFocus ? 0 : 1 * root.px
                                    border.color: root.plateEdge
                                }
                                // The share of its games played: the track, and over it the arc from the top, at least a sliver for a console with games (board lines 66, 67, 130).
                                Shape {
                                    objectName: "plateShare." + plate.index
                                    x: 20 * root.px
                                    y: 20 * root.px
                                    width: 52 * root.px
                                    height: 52 * root.px
                                    preferredRendererType: Shape.CurveRenderer
                                    ShapePath {
                                        objectName: "plateTrack." + plate.index
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
                                        objectName: "plateArc." + plate.index
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
                                            sweepAngle: 360 * root.share(plate.played, plate.count)
                                        }
                                    }
                                }
                                Column {
                                    x: 20 * root.px
                                    width: parent.width - 40 * root.px
                                    y: parent.height - 20 * root.px - height
                                    spacing: 4
                                    Text {
                                        objectName: "plateName." + plate.index
                                        width: parent.width
                                        text: plate.displayName
                                        wrapMode: Text.WordWrap
                                        maximumLineCount: 2
                                        elide: Text.ElideRight
                                        color: Tokens.text
                                        font.pixelSize: 21 * root.px
                                        font.weight: Font.DemiBold
                                        lineHeight: 1.15
                                    }
                                    Text {
                                        objectName: "plateLine." + plate.index
                                        width: parent.width
                                        elide: Text.ElideRight
                                        text: Words.line([plate.year > 0 ? String(plate.year) : "", Words.games(plate.count)])
                                        color: Tokens.muted
                                        font.pixelSize: 16 * root.px
                                    }
                                }
                            }
                        }
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
                                objectName: "consoleEyebrow"
                                width: parent.width
                                elide: Text.ElideRight
                                text: root.shown ? Words.line([root.shown.maker, root.shown.year > 0 ? String(root.shown.year) : ""]) : ""
                                color: root.eyebrowInk
                                font.pixelSize: 18 * root.px
                                font.weight: Font.DemiBold
                                font.letterSpacing: 3.6 * root.px
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
                                font.pixelSize: 46 * root.px
                                font.weight: Font.DemiBold
                                lineHeight: 1.08
                            }
                            Item {
                                width: parent.width
                                height: 14 * root.px
                            }
                            Row {
                                spacing: 10 * root.px
                                width: parent.width
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
                                    width: parent.width - 34 * root.px
                                    wrapMode: Text.WordWrap
                                    // The facts with the played count after the games' count (R-20261115-8).
                                    text: root.shown ? Words.line([Words.facts(root.shown.maker, root.shown.year, root.shown.count, ""), Words.played(root.shown.played), Words.runsOn(root.shown.engineName, root.shown.engineKind)]) : ""
                                    color: Tokens.muted
                                    font.pixelSize: 21 * root.px
                                }
                            }
                            Item {
                                width: parent.width
                                height: 8 * root.px
                            }
                            // Open, the help bar's own A entry as a pill (board line 90).
                            Walkable {
                                id: openButton
                                controller: frame.controller
                                objectName: "consoleOpen"
                                Accessible.name: openWords.text
                                width: (12 + 42 + 12 + 28) * root.px + openWords.implicitWidth
                                height: 64 * root.px
                                onActivated: if (root.shown) Actions.openConsole(root.shown.key)
                                Keys.onLeftPressed: root.reach(root.lastPlate ?? root.plateAt(root.chosenGroup, 0))
                                Rectangle {
                                    objectName: "consoleOpenFill"
                                    anchors.fill: parent
                                    radius: height / 2
                                    color: Tokens.accent
                                }
                                FocusRing {
                                    anchors.fill: parent
                                    radius: height / 2
                                    visible: openButton.activeFocus
                                }
                                PadGlyph {
                                    objectName: "consoleOpenGlyph"
                                    x: 12 * root.px
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 42 * root.px
                                    height: 42 * root.px
                                    family: Device.family
                                    button: PadGlyph.Accept
                                    ink: Tokens.overAccent
                                }
                                Repeater {
                                    id: acceptEntry
                                    model: Help.entries(Help.Console)
                                    Item {
                                        id: entry
                                        required property int button
                                        required property string label
                                        Binding {
                                            when: entry.button === PadGlyph.Accept
                                            target: openWords
                                            property: "text"
                                            value: entry.label
                                        }
                                    }
                                }
                                Text {
                                    id: openWords
                                    objectName: "consoleOpenWords"
                                    x: (12 + 42 + 12) * root.px
                                    height: parent.height
                                    verticalAlignment: Text.AlignVCenter
                                    color: Tokens.overAccent
                                    font.pixelSize: 23 * root.px
                                    font.weight: Font.DemiBold
                                }
                            }
                        }
                    }

                    // No console holds a game: the program's empty words.
                    Column {
                        objectName: "noConsoles"
                        visible: Library.consoles.count === 0
                        y: 537 * root.px
                        width: root.width
                        spacing: 8
                        Text {
                            width: parent.width
                            horizontalAlignment: Text.AlignHCenter
                            text: Words.noConsoles
                            color: Tokens.muted
                            font.pixelSize: 22 * root.px
                        }
                        Text {
                            width: parent.width
                            horizontalAlignment: Text.AlignHCenter
                            text: Words.addFolder
                            color: Tokens.muted
                            font.pixelSize: 22 * root.px
                        }
                    }
                }
            }

            // The chosen maker and where the focused console stands in it, at the foot's right (board line 105).
            Text {
                objectName: "consolePlace"
                visible: root.shown !== null
                x: root.width - root.margin - width
                y: root.foot
                height: 34 * root.px
                verticalAlignment: Text.AlignVCenter
                text: root.shown ? Words.line([root.shown.maker, Words.place(root.shown.groupIndex, root.shown.groupSize)]) : ""
                color: root.makerInk
                font.pixelSize: 21 * root.px
            }
        }
    }
}
