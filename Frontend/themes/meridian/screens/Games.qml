pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import EmuSen.Theme
import "../parts"

// Meridian's Games: the rail collapsed to 64 with the sections' marks and bodies, its focused name floating beside it, and the section's covers in six columns of the cover's own shape; the search and the List card are EmuSen's, drawn over it (meridian.md §4).
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
        required property real coverShape
        required property real consoleHue
        required property bool favourite
        required property string facts
    }
    // A section of the collapsed rail at its rank: the shelves 0 to 2, then the consoles in maker order.
    component Section: RailEntry {
        id: section
        property int rank
        property string key
        property bool isConsole
        readonly property bool current: section.key === Browse.section
        readonly property bool chosen: root.railRank === section.rank
        controller: frame.controller
        faces: root.faceSet
        compact: true
        held: section.current && !section.activeFocus
        onChosenChanged: {
            if (!section.chosen || !root.railHeld)
                return
            section.forceActiveFocus()
            if (section.current)
                return
            if (section.isConsole)
                Actions.openConsole(section.key)
            else
                Actions.openPlace(section.key)
        }
        onCurrentChanged: if (section.current) root.railRank = section.rank
        onActivated: cells.forceActiveFocus()
        onActiveFocusChanged: {
            if (!section.activeFocus)
                return
            root.railRank = section.rank
            root.labelled = section
            rail.contentY = Math.max(0, Math.min(Math.max(rail.contentY, section.y + section.height - rail.height), section.y, Math.max(0, rail.contentHeight - rail.height)))
        }
        Keys.onRightPressed: cells.forceActiveFocus()
    }
    NormalizedCanvas {
        id: canvas
        objectName: "gridCanvas"
        FocusScope {
            id: root
            // First, so the parts read the faces from their first binding.
            Faces {
                id: faces
            }
            objectName: "grid"
            anchors.fill: parent
            focus: true
            readonly property real pixelScale: canvas.unit
            // The faces for the inline components, whose own property of that name would hide the id.
            readonly property Faces faceSet: faces
            readonly property var focusedCell: cells.currentItem
            readonly property bool railHeld: railContent.activeFocus
            property int railRank: -1
            readonly property int lastRank: 2 + Library.consoles.count
            property Section labelled: null
            // The band: x 160 to W - 64; while searching the grid spans 64 to W - 64.
            readonly property real bandX: Browse.searching ? 64 : 160
            readonly property real bandWidth: root.width - 64 - root.bandX
            readonly property real pitch: Browse.searching ? (root.width - 104) / 6 : (root.bandWidth + 24) / 6
            // A console's section takes its cover's shape, at most 208 tall; a shelf of mixed shapes is 156 x 208.
            readonly property real shape: Browse.world.length > 0 && root.focusedCell ? root.focusedCell.coverShape : 0
            readonly property real box: Browse.searching || root.shape <= 0 ? 208 : Math.min(208, Math.round(156 / root.shape))
            readonly property int rowsShown: Browse.searching ? 1 : Math.max(1, Math.floor((580 - 108 + 24) / (root.box + 24)))
            readonly property int rows: Math.ceil(cells.count / 6)
            Backdrop {
                objectName: "gridSky"
                anchors.fill: parent
                glowHue: root.focusedCell ? root.focusedCell.consoleHue : Browse.hue
                glowX: (root.bandX + root.bandWidth / 2) / Math.max(1, root.width)
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
                help: root.railHeld ? Help.entries(Help.Console, { back: true }) : Help.entries(Browse.searched ? Help.SectionSearched : Help.Section)
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
                            width: Math.min(implicitWidth, neighbours.x - 64 - 16 * 3 - 34 - 120 - gridCount.implicitWidth)
                            text: Browse.heading
                            elide: Text.ElideRight
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
                            id: gridCount
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
                        id: neighbours
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

                    // The collapsed rail: hidden while a search narrows the grid.
                    Item {
                        id: railPane
                        objectName: "railPane"
                        anchors.fill: parent
                        opacity: Browse.searching ? 0 : 1
                        visible: railPane.opacity > 0
                        Behavior on opacity {
                            NumberAnimation {
                                duration: Tokens.pan
                                easing.type: Tokens.panCurve
                            }
                        }
                        Glass {
                            objectName: "rail"
                            x: 64
                            y: 96
                            height: 630
                            // Consoles' rail collapsing as the grid opens.
                            NumberAnimation on width {
                                from: 280
                                to: 64
                                duration: Tokens.pan
                                easing.type: Tokens.panCurve
                            }
                        }
                        Flickable {
                            id: rail
                            objectName: "railList"
                            x: 64
                            y: 112
                            width: 64
                            height: 598
                            clip: true
                            contentWidth: 64
                            boundsBehavior: Flickable.StopAtBounds
                            contentHeight: railContent.childrenRect.height
                            property bool fadeTop: rail.contentY > 0.5
                            property bool fadeBottom: rail.contentY + rail.height < rail.contentHeight - 0.5
                            // The software backend draws no layer effect, so there the rows are cut where the GPU fades them.
                            layer.enabled: (rail.fadeTop || rail.fadeBottom) && GraphicsInfo.api !== GraphicsInfo.Software
                            layer.effect: MultiEffect {
                                maskEnabled: true
                                maskSource: fade
                                maskSpreadAtMin: 1
                                maskThresholdMin: 0
                            }
                            Behavior on contentY {
                                NumberAnimation {
                                    duration: Tokens.reel
                                    easing.type: Tokens.reelCurve
                                }
                            }
                            FocusScope {
                                id: railContent
                                objectName: "railContent"
                                width: 64
                                height: railContent.childrenRect.height
                                // With no game to show the rail holds the focus.
                                focus: cells.count === 0 && !Browse.searching
                                Keys.onUpPressed: root.railRank = Math.max(0, root.railRank - 1)
                                Keys.onDownPressed: root.railRank = Math.min(root.lastRank, root.railRank + 1)
                                Repeater {
                                    model: Library.shelves
                                    Section {
                                        id: shelf
                                        required property int index
                                        required key
                                        required name
                                        objectName: "section.shelf." + shelf.index
                                        rank: shelf.index
                                        focus: shelf.current || (Browse.section.length === 0 && shelf.index === 0)
                                        y: shelf.index * 54
                                        mark: Marks.path(shelf.key)
                                    }
                                }
                                Repeater {
                                    model: Library.consoles
                                    Section {
                                        id: consoleSection
                                        required key
                                        required name
                                        required hue
                                        required property string body
                                        required property int groupStart
                                        required property int groupIndex
                                        readonly property int place: consoleSection.groupStart + consoleSection.groupIndex
                                        objectName: "section.console." + consoleSection.place
                                        isConsole: true
                                        rank: 3 + consoleSection.place
                                        focus: consoleSection.current
                                        y: 162 + consoleSection.place * 54
                                        world: consoleSection.body
                                    }
                                }
                            }
                        }
                        RailFade {
                            id: fade
                            x: rail.x
                            y: rail.y
                            width: rail.width
                            height: rail.height
                            atTop: rail.fadeTop
                            atBottom: rail.fadeBottom
                        }
                        Track {
                            objectName: "railTrack"
                            x: 122
                            y: 112
                            width: 2
                            height: 598
                            shown: rail.contentHeight > 0 ? rail.height / rail.contentHeight : 1
                            at: rail.contentHeight > 0 ? rail.contentY / rail.contentHeight : 0
                        }
                    }

                    GridView {
                        id: cells
                        objectName: "gridCells"
                        // Sixteen units of margin about the grid, so a lifted cover's ring is drawn whole.
                        x: root.bandX - 16
                        y: (Browse.searching ? 156 : 108) - 16
                        width: 6 * root.pitch - 24 + 32
                        height: root.rowsShown * (root.box + 24) - 24 + 32
                        leftMargin: 16
                        topMargin: 16
                        cellWidth: root.pitch
                        cellHeight: root.box + 24
                        clip: true
                        focus: cells.count > 0 || Browse.searching
                        keyNavigationWraps: false
                        highlightFollowsCurrentItem: false
                        // The rows move a whole row at a time, the focus kept on the second row once scrolled.
                        readonly property int focusRow: Math.floor(Math.max(0, cells.currentIndex) / 6)
                        readonly property int firstRow: Math.max(0, Math.min(cells.focusRow - 1, root.rows - root.rowsShown))
                        onFirstRowChanged: cells.contentY = cells.originY - cells.topMargin + cells.cellHeight * cells.firstRow
                        Behavior on contentY {
                            NumberAnimation {
                                duration: Tokens.reel
                                easing.type: Tokens.reelCurve
                            }
                        }
                        // Left at the first column goes to the rail, before the grid would wrap to the row above.
                        Keys.onLeftPressed: (event) => {
                            if (!Browse.searching && cells.currentIndex % 6 === 0)
                                railContent.forceActiveFocus()
                            else
                                event.accepted = false
                        }
                        model: Browse.games
                        delegate: Cell {
                            id: cell
                            readonly property bool lit: cell.GridView.isCurrentItem && cells.activeFocus
                            controller: frame.controller
                            objectName: "gridCell." + cell.index
                            Accessible.name: cell.title
                            width: 156
                            height: root.box
                            z: cell.lit ? 1 : 0
                            onActivated: Actions.play(cell.key)
                            onActiveFocusChanged: if (cell.activeFocus) Actions.focus(cell.key)
                            RectangularShadow {
                                x: tile.paintedX
                                y: tile.paintedY - (cell.lit ? 6 : 0)
                                width: tile.paintedWidth
                                height: tile.paintedHeight
                                offset.y: 8
                                blur: 24
                                radius: Tokens.tileRadius
                                color: "#80000000"
                            }
                            CoverTile {
                                id: tile
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
                    Track {
                        objectName: "gridTrack"
                        visible: !Browse.searching && root.rows > root.rowsShown
                        x: root.width - 56
                        y: 108
                        width: 2
                        height: root.rowsShown * (root.box + 24) - 24
                        shown: root.rowsShown / Math.max(1, root.rows)
                        at: cells.firstRow / Math.max(1, root.rows)
                    }
                    Column {
                        x: 160
                        y: 108
                        spacing: 8
                        visible: cells.count === 0 && !Browse.searching
                        Text {
                            objectName: "gridEmpty"
                            text: Words.noGames
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 44
                            lineHeight: 52.8
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            text: Words.addFolder
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 22
                            lineHeight: 26.4
                            lineHeightMode: Text.FixedHeight
                        }
                    }
                    Text {
                        objectName: "gridFocusTitle"
                        visible: !Browse.searching
                        x: 160
                        y: 628
                        width: root.width - 224
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
                        visible: !Browse.searching
                        x: 160
                        y: 670
                        width: root.width - 224
                        text: root.focusedCell ? root.focusedCell.facts : ""
                        elide: Text.ElideRight
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
                    }
                    // The rail's focused name, floating beside it while the rail holds the focus.
                    Item {
                        objectName: "floatingLabel"
                        visible: root.railHeld && root.labelled !== null
                        x: 136
                        y: rail.y + (root.labelled ? root.labelled.y - rail.contentY : 0) + 11
                        width: label.implicitWidth + 32
                        height: 26.4 + 6
                        RectangularShadow {
                            anchors.fill: labelFill
                            offset.y: 8
                            blur: 24
                            radius: 4
                            color: "#80000000"
                        }
                        Rectangle {
                            id: labelFill
                            anchors.fill: parent
                            radius: 4
                            color: Tokens.surfaceRaised
                            opacity: 0.92
                        }
                        Text {
                            id: label
                            x: 16
                            y: 3
                            text: root.labelled ? root.labelled.name : ""
                            color: Tokens.textStrong
                            font.family: faces.regular
                            font.pixelSize: 22
                            lineHeight: 26.4
                            lineHeightMode: Text.FixedHeight
                        }
                    }
                }
            }
        }
    }
}
