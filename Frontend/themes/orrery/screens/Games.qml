pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Orrery's Games: the section's games on an orbit about (W/2, -60) with the focus at its apex, its title and facts centred below; while a search runs, the matches in the reference's one row (orrery.md §4).
ThemeScreen {
    id: screen
    objectName: "gridScreen"
    role: ThemeScreen.Games
    component Match: Walkable {
        required property int index
        required property string key
        required property string title
        required property string cover
        required property real consoleHue
        required property bool favourite
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
            readonly property bool light: Tokens.variant === Tokens.Light
            readonly property color lineInk: root.light ? Qt.rgba(74 / 255, 58 / 255, 40 / 255, 0.5) : Qt.rgba(140 / 255, 150 / 255, 220 / 255, 0.32)
            readonly property var apex: orbit.currentItem
            Backdrop {
                objectName: "gridSky"
                anchors.fill: parent
                glowHue: root.apex ? root.apex.consoleHue : Browse.hue
                glowX: 0.5
                glowY: 500 / 800
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
                            hue: Browse.hue
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
                    // Where the focus is among the section's games, when more than five.
                    Track {
                        objectName: "gridTrack"
                        horizontal: true
                        visible: !Browse.searching && orbit.count > 5
                        x: 64
                        y: 110
                        width: root.width - 128
                        height: 2
                        shown: 5 / Math.max(1, orbit.count)
                        at: Math.max(0, orbit.currentIndex - 2) / Math.max(1, orbit.count)
                    }
                    Item {
                        objectName: "orbitPane"
                        anchors.fill: parent
                        visible: !Browse.searching
                        Scene {
                            objectName: "orbitLine"
                            anchors.fill: parent
                            Shape {
                                anchors.fill: parent
                                preferredRendererType: Shape.CurveRenderer
                                ShapePath {
                                    strokeColor: root.lineInk
                                    strokeWidth: 1
                                    fillColor: "transparent"
                                    PathAngleArc {
                                        centerX: root.width / 2
                                        centerY: -60
                                        radiusX: 600
                                        radiusY: 560
                                        startAngle: 150
                                        sweepAngle: -120
                                    }
                                }
                            }
                        }
                        Orbit {
                            id: orbit
                            objectName: "gridOrbit"
                            x: 0
                            y: 112
                            width: root.width
                            height: 508
                            focus: !Browse.searching
                            model: Browse.games
                            delegate: OrbitCover {
                                id: cell
                                objectName: "gridCell." + cell.index
                                orbit: orbit
                                controller: frame.controller
                                rx: 600
                                ry: 560
                                cy: -60
                                step: 24
                                coverHeight: 200
                                scaleStep: 0.12
                                minScale: 0.64
                                pixelScale: root.pixelScale
                                onActivated: Actions.play(cell.key)
                                onActiveFocusChanged: if (cell.activeFocus) Actions.focus(cell.key)
                            }
                        }
                        // The focus as the program holds it: the theme reports each move (Actions.focus) and the List card moves it too.
                        Binding {
                            orbit.currentIndex: Browse.focusRow
                        }
                        Text {
                            objectName: "gridFocusTitle"
                            visible: root.apex !== null
                            x: 64
                            y: 628
                            width: root.width - 128
                            horizontalAlignment: Text.AlignHCenter
                            text: root.apex ? root.apex.title : ""
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 26
                            lineHeight: 31
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "gridFocusFacts"
                            visible: root.apex !== null
                            x: 64
                            y: 664
                            width: root.width - 128
                            horizontalAlignment: Text.AlignHCenter
                            text: root.apex ? root.apex.facts : ""
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                        Column {
                            x: 64
                            y: 460
                            width: root.width - 128
                            spacing: 8
                            visible: orbit.count === 0
                            Text {
                                objectName: "gridEmpty"
                                width: parent.width
                                horizontalAlignment: Text.AlignHCenter
                                text: Words.noGames
                                color: Tokens.textStrong
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 44
                                lineHeight: 44
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                width: parent.width
                                horizontalAlignment: Text.AlignHCenter
                                text: Words.addFolder
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 22
                                lineHeight: 26.4
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                    }
                    // While a search runs, the matches in the reference's one row: five across at 204, tiles 180 x 240, no lift.
                    GridView {
                        id: matches
                        objectName: "gridCells"
                        visible: Browse.searching
                        x: 64 - 16
                        y: 156 - 16
                        width: 5 * 204 + 32
                        height: 264 - 24 + 32
                        leftMargin: 16
                        topMargin: 16
                        cellWidth: 204
                        cellHeight: 264
                        clip: true
                        focus: Browse.searching
                        keyNavigationWraps: false
                        highlightFollowsCurrentItem: false
                        model: Browse.searching ? Browse.games : null
                        delegate: Match {
                            id: match
                            readonly property bool lit: match.GridView.isCurrentItem && matches.activeFocus
                            controller: frame.controller
                            objectName: "gridMatch." + match.index
                            Accessible.name: match.title
                            width: 180
                            height: 240
                            onActivated: Actions.play(match.key)
                            onActiveFocusChanged: if (match.activeFocus) Actions.focus(match.key)
                            CoverTile {
                                anchors.fill: parent
                                source: match.cover
                                title: match.title
                                hue: match.consoleHue
                                favourite: match.favourite
                                mark: Marks.heart
                                lit: match.lit
                                lifts: false
                                pixelScale: root.pixelScale
                            }
                        }
                    }
                    Binding {
                        when: Browse.searching
                        matches.currentIndex: Browse.focusRow
                    }
                }
            }
        }
    }
}
