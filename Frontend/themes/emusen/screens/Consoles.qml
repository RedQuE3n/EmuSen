pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Shapes
import EmuSen.Theme
import "../parts"

// Choose a console as the orbits: the library's shelves in a row, then each maker's consoles on a half-ellipse, the fewest innermost, three shown and scrolled past; the focused one's words and covers beneath (EmuSen_Qt.md §3.4.2).
ThemeScreen {
    id: screen
    objectName: "consolesScreen"
    role: ThemeScreen.Consoles
    // A shelf in the row under the head: a glass disc with its mark and its name, the walk's first items.
    component Place: Walkable {
        id: place
        required property int index
        required property string key
        required property string name
        required property string line
        required property int count
        readonly property int tileIndex: place.index
        readonly property bool lit: place.activeFocus
        // The disc's centre across, which the orbits' walk goes up to and down from.
        readonly property real centreX: places.x + place.x + 20
        readonly property string fullName: place.name
        readonly property string facts: place.line
        readonly property bool planet: false
        controller: frame.controller
        objectName: "tile." + place.index
        Accessible.name: place.name
        Accessible.description: place.line
        // The section last opened has the focus, else the first console, else the first shelf.
        focus: Browse.section.length > 0 ? place.key === Browse.section : Library.consoles.count === 0 && place.index === 0
        // The canvas's slot, 11.5 units a letter, else the name's own width where that is wider (EmuSen_Qt.md §3.4.2).
        width: 52 + Math.max(Math.round(place.name.length * 11.5), placeName.implicitWidth)
        height: 40
        onCentreXChanged: layout.lead(place.index, place.centreX)
        onActiveFocusChanged: {
            if (!place.activeFocus)
                return
            root.focusedTile = place
            layout.current = place.tileIndex
            Actions.focus(place.key)
        }
        onActivated: Actions.openPlace(place.key)
        Keys.onLeftPressed: layout.stepLeft()
        Keys.onRightPressed: layout.stepRight()
        Keys.onDownPressed: layout.stepDown()
        Rectangle {
            objectName: "placeDisc." + place.index
            width: 40
            height: 40
            radius: 20
            color: Qt.rgba(38 / 255, 38 / 255, 43 / 255, 0.5)
            border.width: 1
            border.color: Qt.rgba(Tokens.muted.r, Tokens.muted.g, Tokens.muted.b, 0.18) // the muted token at 18% (R-20261111-96)
            Glyph {
                anchors.centerIn: parent
                width: 22
                height: 22
                path: Marks.path(place.key)
                ink: night.textStrong
            }
        }
        Rectangle {
            objectName: "placeRing." + place.index
            visible: place.lit
            x: -7
            y: -7
            width: 54
            height: 54
            radius: 27
            color: "transparent"
            border.color: Tokens.accent
            border.width: Tokens.ringThickness
        }
        Text {
            id: placeName
            objectName: "placeName." + place.index
            x: 52
            height: 40
            verticalAlignment: Text.AlignVCenter
            text: place.name
            color: night.textStrong
            font.family: faces.condensed
            font.weight: Font.Medium
            font.pixelSize: 24
            font.letterSpacing: 1.5
            font.capitalization: Font.AllUppercase
        }
    }
    // A console on its maker's orbit: its body (what the pad and the pointer reach), name and count with the engine's mark, at its angle on the orbit's ellipse; the orbit turns and scrolls under it.
    component World: FocusScope {
        id: node
        required property int index
        required property string key
        required property string name
        required property string displayName
        required property string maker
        required property int year
        required property int count
        required property real hue
        required property string body
        required property string engineName
        required property string engineKind
        required property string engineMark
        required property bool libretro
        readonly property int tileIndex: node.index + shelfTiles.count
        readonly property bool lit: hit.activeFocus
        readonly property bool planet: true
        readonly property string fullName: node.displayName
        readonly property string facts: Words.facts(node.maker, node.year, node.count, Words.runsOn(node.engineName, node.engineKind))
        // Its orbit, and the orbit's radii, which move when the orbits scroll.
        readonly property int orbit: layout.orbitOf[node.index] ?? -1
        property real rx: layout.radiiX[node.orbit] ?? 0
        property real ry: layout.radiiY[node.orbit] ?? 0
        readonly property bool shown: layout.shown[node.index] ?? false
        // Where it sits on the orbit, in radians as y grows downward; moved along the arc when the orbit turns.
        property real angle: layout.angles[node.index] ?? Math.PI / 2
        readonly property real cx: layout.centre + node.rx * Math.cos(node.angle)
        readonly property real cy: node.ry * Math.sin(node.angle)
        // The body's height: 88 focused, so its box clears the words of the orbit inside (EmuSen_Qt.md §3.4.2), 72 at rest and while pressed.
        property real size: node.lit && !press.active ? 88 : 72
        objectName: "world." + node.index
        focus: Browse.section.length > 0 ? node.key === Browse.section : node.index === 0
        visible: node.shown || node.opacity > 0
        opacity: node.shown ? 1 : 0
        z: node.lit ? 2 : 1
        // As wide as what it draws, so neighbours on a steep stretch of an orbit do not cover one another.
        // From the top of what it draws (the ring, when focused) to its centre.
        readonly property real above: node.size / 2 + (node.lit ? 7 : 0)
        x: node.cx - width / 2
        y: node.cy - node.above
        width: Math.max(node.size + 14, worldName.width, countRow.width)
        height: node.above + node.size / 2 + 52.4
        Behavior on angle {
            NumberAnimation { duration: Tokens.reel; easing.type: Easing.OutQuad }
        }
        Behavior on rx {
            NumberAnimation { duration: Tokens.pan; easing.type: Easing.OutCubic }
        }
        Behavior on ry {
            NumberAnimation { duration: Tokens.pan; easing.type: Easing.OutCubic }
        }
        Behavior on size {
            NumberAnimation { duration: Tokens.quick; easing.type: Easing.OutQuad }
        }
        Behavior on opacity {
            NumberAnimation { duration: Tokens.fade }
        }
        Walkable {
            id: hit
            controller: frame.controller
            objectName: "tile." + node.tileIndex
            Accessible.name: node.name
            Accessible.description: Words.games(node.count)
            focus: true
            // The body's box; the ring is drawn 7 about it.
            readonly property real margin: node.lit ? 7 : 0
            x: (node.width - width) / 2
            y: node.above - height / 2
            width: node.size
            height: node.size
            onActiveFocusChanged: {
                if (!hit.activeFocus)
                    return
                root.focusedTile = node
                layout.current = node.tileIndex
                Actions.focus(node.key)
            }
            onActivated: Actions.openConsole(node.key)
            Keys.onLeftPressed: layout.stepLeft()
            Keys.onRightPressed: layout.stepRight()
            Keys.onUpPressed: layout.stepUp()
            Keys.onDownPressed: layout.stepDown()
            PointHandler {
                id: press
                acceptedButtons: Qt.LeftButton
            }
            Body {
                id: drawn
                objectName: "consoleBody." + node.index
                world: node.body
                focused: node.lit
                height: node.size
                width: node.size * drawn.tileWidth / Math.max(1, drawn.tileHeight)
                x: (hit.width - width) / 2
            }
            Rectangle {
                objectName: "consoleRing." + node.index
                visible: node.lit
                anchors.fill: parent
                anchors.margins: -hit.margin
                radius: width / 2
                color: "transparent"
                border.color: Tokens.accent
                border.width: Tokens.ringThickness
            }
            Repeater {
                // Each sparkle's side (-1 left of the body, 1 right), its top as a share of the body's height, and its size.
                model: node.lit ? [[-1, 0.08, 13], [1, 0.43, 10], [-1, 0.88, 7]] : []
                Glyph {
                    required property var modelData
                    objectName: "consoleSparkle"
                    x: modelData[0] < 0 ? -4 - width : hit.width
                    y: modelData[1] * node.size
                    width: modelData[2]
                    height: modelData[2]
                    path: Marks.sparkle
                    filled: true
                    ink: night.textStrong
                }
            }
        }
        Text {
            id: worldName
            objectName: "consoleName." + node.index
            x: (node.width - width) / 2
            y: node.above + node.size / 2 + 8
            width: Math.min(implicitWidth, 140)
            text: node.name
            elide: Text.ElideRight
            color: night.textStrong
            font.family: faces.condensed
            font.weight: Font.Medium
            font.pixelSize: 20
            font.letterSpacing: 1.2
            font.capitalization: Font.AllUppercase
            lineHeight: 24
            lineHeightMode: Text.FixedHeight
        }
        Row {
            id: countRow
            x: (node.width - width) / 2
            y: node.above + node.size / 2 + 32
            height: 22.8
            spacing: 6
            Text {
                objectName: "consoleCount." + node.index
                anchors.verticalCenter: parent.verticalCenter
                text: Words.games(node.count)
                color: night.muted
                // 19 units, the smallest whole size over the capitals' floor at 1280 x 720, condensed to stay as wide as the canvas's 17 (EmuSen_Qt.md §3.4.1).
                font.family: faces.condensedRegular
                font.pixelSize: 19
                lineHeight: 22.8
                lineHeightMode: Text.FixedHeight
            }
            // The engine's mark in the console's hue, a Scene as the world's body is (EmuSen_Qt.md §3.4.2).
            Scene {
                anchors.verticalCenter: parent.verticalCenter
                width: 13
                height: 13
                EngineMark {
                    objectName: "consoleEngineMark." + node.index
                    anchors.fill: parent
                    mark: node.engineMark
                    libretro: node.libretro
                    ink: node.libretro ? "#A9A9B2" : Qt.hsla(node.hue / 360, 0.5, 0.72, 1)
                }
            }
        }
    }
    NormalizedCanvas {
        id: canvas
        objectName: "consolesCanvas"
        FocusScope {
            id: root
            objectName: "consoles"
            anchors.fill: parent
            focus: true
            // Canvas units to pixels, for the covers' decoded size.
            readonly property real pixelScale: canvas.unit
            // The focused item, a shelf or a console, and the first shelf (All games), whose count is every game.
            property var focusedTile: null
            property var allGames: null
            OrbitLayout {
                id: layout
                width: root.width
                leading: Library.shelves.count
                model: Library.consoles
                role: "group"
                labelRole: "maker"
            }
            Connections {
                target: Library.consoles
                function onModelReset() { layout.read() }
                function onDataChanged() { layout.read() }
            }
            Connections {
                target: layout
                function onJumped(index) { (index < shelfTiles.count ? shelfTiles.itemAt(index) : consoleTiles.itemAt(index - shelfTiles.count)).forceActiveFocus() }
            }
            Sky {
                id: sky
                objectName: "consolesSky"
                anchors.fill: parent
                glowHue: root.focusedTile && root.focusedTile.planet ? root.focusedTile.hue : -1
                glowX: root.focusedTile && root.focusedTile.planet ? root.focusedTile.cx / root.width : 0.5
                glowY: root.focusedTile && root.focusedTile.planet ? root.focusedTile.cy / root.height : 0.5
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                padFamily: Device.family
                helpFont: Qt.font({ family: faces.regular, pixelSize: 20 })
                helpSpacing: 40
                helpGlyphGap: 10
                focus: true
                controller.navigationShown: false
                controller.sections: layout.sections
                controller.selectedSection: layout.section
                help: Help.entries(Help.Console, {group: true})
                Connections {
                    target: frame.controller
                    function onBackDeclined() { Actions.back() }
                    function onActionInvoked(key) { Actions.act(key) }
                    function onSectionChosen(key) { layout.jump(key) }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    Text {
                        id: title
                        objectName: "consolesTitle"
                        x: 64
                        y: 40
                        text: Words.gamesHeading
                        color: night.textStrong
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 32
                        font.letterSpacing: 3
                        lineHeight: 38.4
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "consolesSubtitle"
                        x: title.x + title.contentWidth + 20
                        y: 52
                        text: Words.chooseAConsole
                        color: night.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "consolesSummary"
                        x: root.width - 64 - width
                        y: 52
                        text: Words.summary(Library.consoles.count, root.allGames ? root.allGames.count : 0)
                        color: night.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Item {
                        // The orbits drawn from 160° to 20° and cut at y 584, above the foot's words.
                        objectName: "orbitLines"
                        width: root.width
                        height: 584
                        clip: true
                        // The lines are drawn behind the worlds, as the sky is, not beside them.
                        readonly property bool backdrop: true
                        Repeater {
                            model: layout.orbitCount
                            Shape {
                                id: line
                                required property int index
                                property real rx: layout.radiiX[line.index] ?? 0
                                property real ry: layout.radiiY[line.index] ?? 0
                                objectName: "orbit." + line.index
                                width: root.width
                                height: 584
                                visible: line.opacity > 0
                                opacity: (layout.orbitShown[line.index] ?? false) ? (layout.focusedOrbit === line.index ? 1 : 0.6) : 0
                                preferredRendererType: Shape.CurveRenderer
                                Behavior on rx {
                                    NumberAnimation { duration: Tokens.pan; easing.type: Easing.OutCubic }
                                }
                                Behavior on ry {
                                    NumberAnimation { duration: Tokens.pan; easing.type: Easing.OutCubic }
                                }
                                Behavior on opacity {
                                    NumberAnimation { duration: Tokens.quick }
                                }
                                ShapePath {
                                    fillColor: "transparent"
                                    strokeColor: Qt.rgba(140 / 255, 150 / 255, 220 / 255, 0.32)
                                    strokeWidth: 1
                                    PathAngleArc {
                                        centerX: layout.centre
                                        centerY: 0
                                        radiusX: line.rx
                                        radiusY: line.ry
                                        startAngle: 20
                                        sweepAngle: 140
                                    }
                                }
                            }
                        }
                    }
                    Repeater {
                        // Each shown orbit's maker, right-aligned inside its window's end, with the count when the orbit turns.
                        model: layout.orbitCount
                        Row {
                            id: label
                            required property int index
                            readonly property bool turning: layout.turning[label.index] ?? false
                            objectName: "makerLabel." + label.index
                            visible: layout.orbitShown[label.index] ?? false
                            // The focused orbit's maker brighter than the rest, by ink rather than opacity, so the rest keep 4.5:1 over the crowd.
                            readonly property color ink: layout.focusedOrbit === label.index ? night.text : night.muted
                            x: (layout.labelRights[label.index] ?? 0) - width
                            // The 19-unit line grows upward, its foot where the layout's 20-unit box ends (EmuSen_Qt.md §3.4.2).
                            y: (layout.labelTops[label.index] ?? 0) + 20 - height
                            height: 22.8
                            spacing: 5
                            Text {
                                text: layout.labels[label.index] ?? ""
                                color: label.ink
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 19
                                font.letterSpacing: 2
                                font.capitalization: Font.AllUppercase
                                lineHeight: 22.8
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                visible: label.turning
                                text: Words.between
                                color: label.ink
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 19
                                lineHeight: 22.8
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                visible: label.turning
                                text: layout.sizes[label.index] ?? 0
                                color: label.ink
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 19
                                font.letterSpacing: 2
                                lineHeight: 22.8
                                lineHeightMode: Text.FixedHeight
                            }
                        }
                    }
                    Repeater {
                        // Three dots past a turning orbit's window's left end, where earlier worlds wait.
                        model: layout.orbitCount
                        Row {
                            id: before
                            required property int index
                            objectName: "moreBefore." + before.index
                            visible: !isNaN(layout.beforeXs[before.index] ?? NaN) && (layout.orbitShown[before.index] ?? false)
                            x: (layout.beforeXs[before.index] ?? 0) - 10
                            y: (layout.beforeYs[before.index] ?? 0) - 2
                            spacing: 4
                            Repeater {
                                model: 3
                                Rectangle {
                                    width: 3
                                    height: 3
                                    radius: 1.5
                                    color: night.muted
                                }
                            }
                        }
                    }
                    Repeater {
                        // And past its right end, where later worlds wait.
                        model: layout.orbitCount
                        Row {
                            id: after
                            required property int index
                            objectName: "moreAfter." + after.index
                            visible: !isNaN(layout.afterXs[after.index] ?? NaN) && (layout.orbitShown[after.index] ?? false)
                            x: (layout.afterXs[after.index] ?? 0) - 10
                            y: (layout.afterYs[after.index] ?? 0) - 2
                            spacing: 4
                            Repeater {
                                model: 3
                                Rectangle {
                                    width: 3
                                    height: 3
                                    radius: 1.5
                                    color: night.muted
                                }
                            }
                        }
                    }
                    Row {
                        id: places
                        objectName: "places"
                        x: (root.width - width) / 2
                        y: 100
                        spacing: 48
                        Repeater {
                            id: shelfTiles
                            model: Library.shelves
                            onItemAdded: (index, item) => root.allGames = index === 0 ? item : root.allGames
                            Place {}
                        }
                    }
                    Repeater {
                        id: consoleTiles
                        model: Library.consoles
                        World {}
                    }
                    Text {
                        objectName: "noConsoles"
                        visible: Library.consoles.count === 0
                        y: 358
                        width: root.width
                        horizontalAlignment: Text.AlignHCenter
                        text: Words.noConsoles
                        color: night.muted
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "consoleDetailName"
                        x: 64
                        // The face's own line box, 53, its baseline where the canvas's is (R-20261111-94, EmuSen_Qt.md §3.4.2).
                        y: 607.6
                        width: coverRow.x - 96
                        text: root.focusedTile ? root.focusedTile.fullName : ""
                        elide: Text.ElideRight
                        color: night.textStrong
                        font.family: faces.condensed
                        font.weight: Font.Medium
                        font.pixelSize: 44
                        font.letterSpacing: 2
                        font.capitalization: Font.AllUppercase
                    }
                    Text {
                        objectName: "consoleDetailFacts"
                        x: 64
                        y: 664
                        width: coverRow.x - 96
                        text: root.focusedTile ? root.focusedTile.facts : ""
                        elide: Text.ElideRight
                        color: night.muted
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
                    }
                    EngineLine {
                        objectName: "consoleDetailEngine"
                        x: 64
                        y: 696
                        height: 20
                        visible: root.focusedTile ? root.focusedTile.planet : false
                        name: root.focusedTile && root.focusedTile.planet ? root.focusedTile.engineName : ""
                        mark: root.focusedTile && root.focusedTile.planet ? root.focusedTile.engineMark : ""
                        libretro: root.focusedTile && root.focusedTile.planet ? root.focusedTile.libretro : false
                        hue: root.focusedTile && root.focusedTile.planet ? root.focusedTile.hue : 0
                        family: faces.condensed
                    }
                    CoverRow {
                        id: coverRow
                        objectName: "coverRow"
                        pixelScale: root.pixelScale
                        x: root.width - 64 - width
                        y: 616
                        height: 100
                        games: Library.firstOf(root.focusedTile ? root.focusedTile.key : "", 3)
                    }
                }
            }
            Faces {
                id: faces
            }
            // The world is the night in both variants, its words the dark variant's inks (decided 2026-11-09).
            Night {
                id: night
            }
        }
    }
}
