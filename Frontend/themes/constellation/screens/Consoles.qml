pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import EmuSen.Theme
import "../parts"

// Choose a console, "Figures": each maker a constellation, its consoles the stars of a zigzag figure by count joined by the chart's lines, the three shelves the Pointers closing the first row, and the focused one's body, words and covers in a key of glass (constellation.md §3).
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
            readonly property real pixelScale: canvas.unit
            readonly property bool light: Tokens.variant === Tokens.Light
            readonly property bool resting: !Tokens.reducedMotion && !(Device.battery >= 0 && !Device.charging)
            readonly property int count: Library.consoles.count
            // The focused node, a console's body or a Pointer, and its hue (-1 for a Pointer).
            property var focused: null
            // How many nodes the Repeaters have made, so a lookup by index is read again as each is made.
            property int made: 0
            readonly property var allShelf: root.made > 0 && pointers.count > 0 ? pointers.itemAt(0) : null
            readonly property real focusHue: root.focused ? root.focused.hue : -1
            readonly property color restInk: root.light ? "#1F2A44" : Qt.hsla(220 / 360, 0.6, 0.8, 1)
            readonly property color litInk: root.focusHue >= 0 ? Qt.hsla(root.focusHue / 360, root.light ? 0.6 : 0.8, root.light ? 0.35 : 0.7, 1) : Tokens.accent
            property real breath: 0
            SequentialAnimation on breath {
                running: root.resting && root.visible
                loops: Animation.Infinite
                NumberAnimation { from: 0; to: 1; duration: 2000; easing.type: Easing.InOutSine }
                NumberAnimation { from: 1; to: 0; duration: 2000; easing.type: Easing.InOutSine }
            }
            readonly property real restOpacity: 0.08 + 0.04 * root.breath
            // The camera on the focused row, clamped to the chart: 468 units of the layout shown from y 96, with 8 more at each end for a focused body's label.
            readonly property real camera: root.focused && layout.scrolls ? Math.max(0, Math.min(root.focused.cy - 234, layout.contentHeight - 468)) : 0
            onCameraChanged: chartView.contentY = root.camera
            FigureLayout {
                id: layout
                width: root.width
                model: Library.consoles
                role: "group"
            }
            Connections {
                target: Library.consoles
                function onModelReset() { layout.read() }
            }
            Backdrop {
                objectName: "consolesSky"
                anchors.fill: parent
                resting: root.resting
                glowHue: root.focusHue
                glowX: root.focused ? root.focused.cx / root.width : 0.5
                glowY: root.focused ? (root.focused.cy + 96 - chartView.contentY) / root.height : 0.4
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
                help: Help.entries(Help.Console, { group: true })
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
                    Text {
                        id: title
                        objectName: "consolesTitle"
                        x: 64
                        y: 40
                        text: Words.gamesHeading
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
                        objectName: "consolesSubtitle"
                        x: title.x + title.contentWidth + 20
                        y: 52
                        text: Words.chooseAConsole
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "consolesSummary"
                        x: root.width - 64 - width
                        y: 52
                        text: Words.summary(root.count, root.allShelf ? root.allShelf.count : 0)
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    // The chart, 96 to 564 and 8 past each end where a focused body's label steps clear of its ring, scrolled under a clamped camera on the focused row and faded 24 at an edge it runs past.
                    Flickable {
                        id: chartView
                        objectName: "chart"
                        y: 88
                        width: root.width
                        height: 484
                        clip: true
                        contentWidth: root.width
                        contentHeight: Math.max(484, layout.contentHeight + 16)
                        boundsBehavior: Flickable.StopAtBounds
                        property bool fadeTop: chartView.contentY > 0.5
                        property bool fadeBottom: chartView.contentY + chartView.height < chartView.contentHeight - 0.5
                        // The software backend draws no layer effect, so there the rows are cut where the GPU fades them.
                        layer.enabled: (chartView.fadeTop || chartView.fadeBottom) && GraphicsInfo.api !== GraphicsInfo.Software
                        layer.effect: MultiEffect {
                            maskEnabled: true
                            maskSource: fade
                            maskSpreadAtMin: 1
                            maskThresholdMin: 0
                        }
                        Behavior on contentY {
                            NumberAnimation {
                                duration: Tokens.pan
                                easing.type: Tokens.panCurve
                            }
                        }
                        // Content coordinates: the layout's y less the chart's top, 96.
                        Item {
                            id: content
                            y: 8
                            width: root.width
                            height: chartView.contentHeight - 16
                            Scene {
                                id: lines
                                objectName: "chartLines"
                                anchors.fill: parent
                                // A figure's line from each body to the one before it, and the Pointers' triangle.
                                Repeater {
                                    model: root.count
                                    ChartLine {
                                        id: link
                                        required property int index
                                        readonly property int prior: layout.prior[link.index] ?? -1
                                        readonly property var a: root.made > 0 && bodies.count > link.index ? bodies.itemAt(link.index) : null
                                        readonly property var b: root.made > 0 && link.prior >= 0 && bodies.count > link.prior ? bodies.itemAt(link.prior) : null
                                        readonly property bool lit: root.focused !== null && (root.focused === link.a || root.focused === link.b)
                                        visible: link.a !== null && link.b !== null
                                        x1: link.a ? link.a.cx : 0
                                        y1: link.a ? link.a.cy : 0
                                        x2: link.b ? link.b.cx : 0
                                        y2: link.b ? link.b.cy : 0
                                        ink: link.lit ? root.litInk : root.restInk
                                        opacity: link.lit ? 0.45 : root.restOpacity
                                    }
                                }
                                Repeater {
                                    model: pointers.count === 3 ? 3 : 0
                                    ChartLine {
                                        id: side
                                        required property int index
                                        readonly property var a: root.made > 0 ? pointers.itemAt(side.index) : null
                                        readonly property var b: root.made > 0 ? pointers.itemAt((side.index + 1) % 3) : null
                                        readonly property bool lit: root.focused !== null && (root.focused === side.a || root.focused === side.b)
                                        visible: side.a !== null && side.b !== null
                                        x1: side.a ? side.a.cx : 0
                                        y1: side.a ? side.a.cy : 0
                                        x2: side.b ? side.b.cx : 0
                                        y2: side.b ? side.b.cy : 0
                                        ink: side.lit ? root.litInk : root.restInk
                                        opacity: side.lit ? 0.45 : root.restOpacity
                                    }
                                }
                                // The walk's other edges from the focus, dashed: where the pad goes that no chart line draws.
                                Repeater {
                                    model: 4
                                    ChartLine {
                                        id: walkEdge
                                        required property int index
                                        readonly property var to: root?.focused ? (walkEdge.index === 0 ? root.focused.leftItem : (walkEdge.index === 1 ? root.focused.rightItem : (walkEdge.index === 2 ? root.focused.upItem : root.focused.downItem))) : null
                                        readonly property bool drawn: !!walkEdge.to && (root?.focused?.isPointer ? !walkEdge.to.isPointer : walkEdge.to.figure !== root?.focused?.figure)
                                        visible: walkEdge.drawn
                                        dashed: true
                                        x1: root?.focused ? root.focused.cx : 0
                                        y1: root?.focused ? root.focused.cy : 0
                                        x2: walkEdge.to ? walkEdge.to.cx : 0
                                        y2: walkEdge.to ? walkEdge.to.cy : 0
                                        ink: root.litInk
                                        opacity: 0.45
                                    }
                                }
                            }
                            // Each maker's chart label, right-aligned in the zone at its figure's left, on its first line's centre.
                            Repeater {
                                model: Library.consoles
                                Row {
                                    id: makerLabel
                                    required property string maker
                                    required property int group
                                    required property int groupIndex
                                    required property int groupSize
                                    objectName: "maker." + makerLabel.group
                                    visible: makerLabel.groupIndex === 0
                                    x: (layout.figureXs[makerLabel.group] ?? 0) + layout.zone - 12 - width
                                    y: (layout.figureYs[makerLabel.group] ?? 0) - 96 - 10
                                    spacing: 6
                                    Text {
                                        text: makerLabel.maker
                                        color: Tokens.muted
                                        font.family: faces.condensed
                                        font.weight: Font.Medium
                                        font.pixelSize: 19
                                        font.letterSpacing: 2
                                        font.capitalization: Font.AllUppercase
                                        lineHeight: 22.8
                                        lineHeightMode: Text.FixedHeight
                                    }
                                    Text {
                                        visible: makerLabel.groupSize > 1
                                        text: Words.between
                                        color: Tokens.muted
                                        font.family: faces.condensed
                                        font.pixelSize: 19
                                        lineHeight: 22.8
                                        lineHeightMode: Text.FixedHeight
                                    }
                                    Text {
                                        visible: makerLabel.groupSize > 1
                                        text: makerLabel.groupSize
                                        color: Tokens.muted
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
                                id: bodies
                                model: Library.consoles
                                onItemAdded: root.made = root.made + 1
                                onItemRemoved: root.made = root.made - 1
                                ChartBody {
                                    id: node
                                    controller: frame.controller
                                    layout: layout
                                    bodies: bodies
                                    pointers: pointers
                                    faces: faces
                                    consoleCount: root.count
                                    made: root.made
                                    viewTop: chartView.contentY
                                    // The console left last has the focus, else the first.
                                    focus: Browse.section.length > 0 ? node.key === Browse.section : node.index === 0
                                    onActiveFocusChanged: {
                                        root.focused = node.activeFocus ? node : (root.focused === node ? null : root.focused)
                                        if (node.activeFocus) Actions.focus(node.key)
                                    }
                                    onActivated: Actions.openConsole(node.key)
                                }
                            }
                            Repeater {
                                id: pointers
                                model: Library.shelves
                                onItemAdded: root.made = root.made + 1
                                onItemRemoved: root.made = root.made - 1
                                ChartPointer {
                                    id: pointer
                                    controller: frame.controller
                                    layout: layout
                                    bodies: bodies
                                    pointers: pointers
                                    faces: faces
                                    consoleCount: root.count
                                    made: root.made
                                    viewTop: chartView.contentY
                                    focus: Browse.section.length > 0 ? pointer.key === Browse.section : root.count === 0 && pointer.index === 0
                                    onActiveFocusChanged: {
                                        root.focused = pointer.activeFocus ? pointer : (root.focused === pointer ? null : root.focused)
                                        if (pointer.activeFocus) Actions.focus(pointer.key)
                                    }
                                    onActivated: Actions.openPlace(pointer.key)
                                }
                            }
                        }
                    }
                    ChartFade {
                        id: fade
                        x: chartView.x
                        y: chartView.y
                        width: chartView.width
                        height: chartView.height
                        atTop: chartView.fadeTop
                        atBottom: chartView.fadeBottom
                    }
                    Rectangle {
                        objectName: "chartTrack"
                        visible: layout.scrolls
                        x: root.width - 56
                        y: 96
                        width: 2
                        height: 468
                        radius: 1
                        color: Tokens.border
                        opacity: 0.35
                    }
                    Rectangle {
                        objectName: "chartThumb"
                        visible: layout.scrolls
                        x: root.width - 56
                        y: 96 + (468 - height) * (chartView.contentHeight > chartView.height ? chartView.contentY / (chartView.contentHeight - chartView.height) : 0)
                        width: 2
                        height: Math.max(24, 468 * chartView.height / Math.max(chartView.height, chartView.contentHeight))
                        radius: 1
                        color: Tokens.border
                    }
                    Text {
                        objectName: "noConsoles"
                        visible: root.count === 0
                        y: 378
                        width: root.width
                        horizontalAlignment: Text.AlignHCenter
                        text: Words.noConsoles
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
                    }
                    ConsoleKey {
                        objectName: "consoleKey"
                        x: 64
                        y: 596
                        width: root.width - 128
                        height: 124
                        node: root.focused
                        faces: faces
                        pixelScale: root.pixelScale
                    }
                }
            }
            Faces {
                id: faces
            }
        }
    }
}
