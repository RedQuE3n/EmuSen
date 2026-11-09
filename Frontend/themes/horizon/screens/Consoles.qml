pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Horizon's Choose a console: the focused console risen as a world at its body's size, its words, engine line and three covers in the sky; the three places, then every console grouped by maker on the land under the clamped camera (EmuSen_Qt.md §3.4.2).
ThemeScreen {
    id: screen
    objectName: "horizonConsoles"
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
            // The places, then the consoles in maker order: a console's place is the places' count plus groupStart plus groupIndex.
            readonly property int places: Library.shelves.count
            readonly property int slots: root.places + Library.consoles.count
            readonly property real placesWidth: Math.max(0, root.places * 168 - 28)
            property int groups: 0
            property int allGames: 0
            property int focusSlot: 0
            property var focused: null
            // The first place of the group before the focused one, given by that group's last machine.
            property int previousFirst: 0
            readonly property bool onPlace: root.focused ? root.focused.kind === 2 : true
            readonly property int groupFirst: root.focused && !root.onPlace ? root.focused.groupFirst : 0
            readonly property int groupEnd: root.focused && !root.onPlace ? root.focused.groupEnd : root.places
            readonly property int nextFirst: root.groupEnd < root.slots ? root.groupEnd : root.focusSlot
            readonly property real hue: root.focused && !root.onPlace ? root.focused.hue : -1
            readonly property real wordsWidth: Math.min(820, root.width - 480)
            readonly property bool night: Tokens.variant === Tokens.Dark
            Backdrop {
                anchors.fill: parent
                hue: root.hue
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
                        id: heading
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
                        objectName: "consolesChoose"
                        x: heading.x + heading.width + 20
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
                        text: Words.summary(Library.consoles.count, root.allGames)
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    World {
                        x: root.width - 364
                        y: 140
                        key: root.focused ? root.focused.key : ""
                        place: root.onPlace
                        world: root.focused ? root.focused.world : ""
                        mark: root.focused ? root.focused.mark : ""
                        hue: root.hue
                        pixelScale: root.pixelScale
                    }
                    Column {
                        objectName: "consolesWords"
                        x: 64
                        y: 408 - height
                        width: root.wordsWidth
                        spacing: 8
                        Text {
                            objectName: "consolesEyebrow"
                            visible: root.onPlace
                            text: Words.places
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            font.letterSpacing: 2
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "consolesName"
                            width: parent.width
                            text: root.focused ? root.focused.title : ""
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 44
                            font.letterSpacing: 2
                            font.capitalization: Font.AllUppercase
                            lineHeight: 52.8
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "consolesFacts"
                            width: parent.width
                            text: root.focused ? root.focused.facts : ""
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: root.onPlace ? 26 : 22
                            lineHeight: root.onPlace ? 31.2 : 26.4
                            lineHeightMode: Text.FixedHeight
                        }
                        Row {
                            objectName: "engineLine"
                            visible: !root.onPlace
                            spacing: 6
                            height: 20.4
                            readonly property bool libretro: root.focused?.libretro ?? false
                            readonly property color ink: engine.libretro ? (root.night ? "#C9C9D0" : Tokens.muted) : Qt.hsla(Math.max(0, root.hue) / 360, root.night ? 0.85 : 0.6, root.night ? 0.8 : 0.3, 1)
                            id: engine
                            EngineMark {
                                anchors.verticalCenter: parent.verticalCenter
                                width: 15
                                height: 15
                                mark: root.focused?.engineMark ?? ""
                                libretro: engine.libretro
                                ink: engine.ink
                            }
                            Text {
                                objectName: "engineName"
                                anchors.verticalCenter: parent.verticalCenter
                                width: Math.min(150, implicitWidth)
                                text: root.focused?.engineName ?? ""
                                elide: Text.ElideRight
                                color: engine.ink
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 17
                                font.letterSpacing: 1.5
                                font.capitalization: Font.AllUppercase
                            }
                            Rectangle {
                                objectName: "engineTag"
                                anchors.verticalCenter: parent.verticalCenter
                                width: tag.implicitWidth + 10
                                height: 22
                                radius: Tokens.rowRadius
                                color: engine.libretro ? "transparent" : Qt.hsla(Math.max(0, root.hue) / 360, 0.6, 0.5, 0.14)
                                border.width: 1
                                border.color: engine.libretro ? (root.night ? "#6E6E78" : "#8C8C8C") : Qt.hsla(Math.max(0, root.hue) / 360, 0.45, root.night ? 0.52 : 0.4, 1)
                                Text {
                                    id: tag
                                    anchors.centerIn: parent
                                    text: engine.libretro ? Words.libretroTag : Words.emusenTag
                                    color: engine.libretro ? (root.night ? "#A9A9B2" : Tokens.muted) : Qt.hsla(Math.max(0, root.hue) / 360, root.night ? 0.7 : 0.6, root.night ? 0.82 : 0.3, 1)
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 17
                                    font.letterSpacing: 0.8
                                }
                            }
                        }
                        // Three covers whole, 100 tall and at most 150 wide, 12 apart, their feet on the block's foot.
                        Row {
                            objectName: "coverRow"
                            topPadding: 4
                            spacing: 12
                            height: 104
                            Repeater {
                                model: Library.firstOf(root.focused ? root.focused.key : "", 3)
                                Cover {
                                    id: shown
                                    required property int index
                                    required property string cover
                                    required property string title
                                    required property real consoleHue
                                    required property real coverShape
                                    required property bool favourite
                                    objectName: "rowCover." + shown.index
                                    y: 4
                                    width: Math.min(150, 100 * shown.coverShape)
                                    height: 100
                                    source: shown.cover
                                    title: shown.title
                                    hue: shown.consoleHue
                                    favourite: shown.favourite
                                    pixelScale: root.pixelScale
                                }
                            }
                        }
                    }
                    HorizonLine {
                        width: root.width
                        total: strip.total
                        off: strip.off
                    }
                    Strip {
                        id: strip
                        width: root.width
                        total: Library.consoles.count > 0 ? root.placesWidth + 24 * root.groups + 188 * Library.consoles.count : root.placesWidth + 52 + noConsoles.width
                        focusX: root.focused ? root.focused.stripX : 0
                        focusWidth: root.focused ? root.focused.width : 0
                        onStepped: (by) => root.focusSlot = Math.max(0, Math.min(root.slots - 1, root.focusSlot + by))
                        Repeater {
                            id: shelves
                            model: Library.shelves
                            StripItem {
                                id: shelf
                                required property int index
                                required key
                                required line
                                required property string name
                                required property int count
                                readonly property int slot: shelf.index
                                readonly property bool chosen: root.focusSlot === shelf.slot
                                controller: frame.controller
                                strip: strip
                                objectName: "shelf." + shelf.index
                                focus: Browse.section.length > 0 ? shelf.key === Browse.section : Library.consoles.count === 0 && shelf.index === 0
                                kind: 2
                                title: shelf.name
                                facts: shelf.line
                                mark: Marks.path(shelf.key)
                                stripX: shelf.index * 168
                                x: shelf.stripX
                                y: strip.foot - height
                                width: 140
                                height: 88
                                boxX: 42
                                boxWidth: 56
                                boxHeight: 56
                                round: true
                                Accessible.name: shelf.name
                                Accessible.description: shelf.line
                                onActivated: Actions.openPlace(shelf.key)
                                onChosenChanged: if (shelf.chosen) shelf.forceActiveFocus()
                                onWalked: (key) => root.focusSlot = key === Qt.Key_Left ? Math.max(0, root.focusSlot - 1) : (key === Qt.Key_Right ? Math.min(root.slots - 1, root.focusSlot + 1) : (key === Qt.Key_PageDown ? root.nextFirst : (key === Qt.Key_PageUp ? 0 : root.focusSlot)))
                                onActiveFocusChanged: {
                                    if (!shelf.activeFocus)
                                        return
                                    root.focusSlot = shelf.slot
                                    root.focused = shelf
                                    Actions.focus(shelf.key)
                                }
                                Binding {
                                    root.allGames: shelf.count
                                    when: shelf.index === 0
                                }
                                PlaceDisc {
                                    x: 42
                                    mark: shelf.mark
                                }
                                Text {
                                    objectName: "shelfName." + shelf.index
                                    y: 64
                                    width: 140
                                    horizontalAlignment: Text.AlignHCenter
                                    text: shelf.name
                                    elide: Text.ElideRight
                                    color: Tokens.textStrong
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 20
                                    font.letterSpacing: 1.2
                                    font.capitalization: Font.AllUppercase
                                    lineHeight: 24
                                    lineHeightMode: Text.FixedHeight
                                }
                            }
                        }
                        Text {
                            id: noConsoles
                            objectName: "noConsoles"
                            visible: Library.consoles.count === 0
                            x: root.placesWidth + 52
                            y: strip.foot - height
                            text: Words.noConsoles
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 20
                            lineHeight: 24
                            lineHeightMode: Text.FixedHeight
                        }
                        Repeater {
                            model: Library.consoles
                            StripItem {
                                id: machine
                                required key
                                required property string name
                                required property string displayName
                                required property string maker
                                required property int year
                                required property int count
                                required property string body
                                required property string engineName
                                required property string engineKind
                                required property string engineMark
                                required property bool libretro
                                required property int group
                                required property int groupIndex
                                required property int groupSize
                                required property int groupStart
                                required hue
                                readonly property int slot: root.places + machine.groupStart + machine.groupIndex
                                readonly property int groupFirst: root.places + machine.groupStart
                                readonly property int groupEnd: root.places + machine.groupStart + machine.groupSize
                                readonly property bool chosen: root.focusSlot === machine.slot
                                controller: frame.controller
                                strip: strip
                                objectName: "machine." + machine.key
                                focus: Browse.section.length > 0 ? machine.key === Browse.section : machine.slot === root.places
                                kind: 3
                                title: machine.displayName
                                world: machine.body
                                facts: Words.facts(machine.maker, machine.year, machine.count, Words.runsOn(machine.engineName, machine.engineKind))
                                stripX: root.placesWidth + 28 + 24 * (machine.group + 1) + 188 * (machine.groupStart + machine.groupIndex)
                                x: machine.stripX
                                y: strip.foot - height
                                width: 160
                                height: 134
                                boxX: 40
                                boxWidth: 80
                                boxHeight: 80
                                round: true
                                Accessible.name: machine.name
                                onActivated: Actions.openConsole(machine.key)
                                onChosenChanged: if (machine.chosen) machine.forceActiveFocus()
                                onWalked: (key) => root.focusSlot = key === Qt.Key_Left ? Math.max(0, root.focusSlot - 1) : (key === Qt.Key_Right ? Math.min(root.slots - 1, root.focusSlot + 1) : (key === Qt.Key_PageDown ? root.nextFirst : (key === Qt.Key_PageUp ? (machine.group === 0 ? 0 : root.previousFirst) : root.focusSlot)))
                                onActiveFocusChanged: {
                                    if (!machine.activeFocus)
                                        return
                                    root.focusSlot = machine.slot
                                    root.focused = machine
                                    Actions.focus(machine.key)
                                }
                                Binding {
                                    root.groups: machine.group + 1
                                    when: machine.groupIndex === 0 && machine.groupStart + machine.groupSize === Library.consoles.count
                                }
                                Binding {
                                    root.previousFirst: machine.groupFirst
                                    when: machine.slot === root.groupFirst - 1 && root.groupFirst > root.places
                                }
                                Body {
                                    id: planet
                                    objectName: "consoleBody." + machine.key
                                    world: machine.body
                                    focused: machine.lit
                                    height: 80
                                    width: 80 * planet.tileWidth / Math.max(1, planet.tileHeight)
                                    x: 80 - width / 2
                                }
                                Repeater {
                                    // Each sparkle's side (-1 left of the body, 1 right), its top as a share of the body's height, and its size.
                                    model: machine.lit ? [[-1, 0.08, 13], [1, 0.43, 10], [-1, 0.88, 7]] : []
                                    Glyph {
                                        required property var modelData
                                        objectName: "consoleSparkle"
                                        x: modelData[0] < 0 ? 40 - width - 2 : 122
                                        y: modelData[1] * 80
                                        width: modelData[2]
                                        height: modelData[2]
                                        path: Marks.sparkle
                                        filled: true
                                        ink: Tokens.textStrong
                                    }
                                }
                                Text {
                                    objectName: "consoleName." + machine.key
                                    y: 88
                                    width: 160
                                    horizontalAlignment: Text.AlignHCenter
                                    text: machine.name
                                    elide: Text.ElideRight
                                    color: Tokens.textStrong
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 22
                                    font.letterSpacing: 1.2
                                    font.capitalization: Font.AllUppercase
                                    lineHeight: 26.4
                                    lineHeightMode: Text.FixedHeight
                                }
                                Text {
                                    objectName: "consoleCount." + machine.key
                                    y: 114.4
                                    width: 160
                                    horizontalAlignment: Text.AlignHCenter
                                    text: Words.games(machine.count)
                                    color: Tokens.muted
                                    font.family: faces.regular
                                    font.pixelSize: 17
                                    lineHeight: 20.4
                                    lineHeightMode: Text.FixedHeight
                                }
                            }
                        }
                        // Each maker's label rides the strip under the line, over its first body: the maker and its count.
                        Repeater {
                            model: Library.consoles
                            Row {
                                id: label
                                required property string maker
                                required property int group
                                required property int groupIndex
                                required property int groupSize
                                required property int groupStart
                                objectName: "maker." + label.group
                                visible: label.groupIndex === 0
                                x: root.placesWidth + 28 + 24 * (label.group + 1) + 188 * label.groupStart
                                y: 12
                                spacing: 6
                                Text {
                                    visible: label.maker.length > 0
                                    text: label.maker
                                    color: Tokens.muted
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 17
                                    font.letterSpacing: 2
                                    font.capitalization: Font.AllUppercase
                                    lineHeight: 20
                                    lineHeightMode: Text.FixedHeight
                                }
                                Text {
                                    visible: label.maker.length > 0
                                    text: Words.between
                                    color: Tokens.muted
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 17
                                    lineHeight: 20
                                    lineHeightMode: Text.FixedHeight
                                }
                                Text {
                                    text: label.groupSize
                                    color: Tokens.muted
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 17
                                    font.letterSpacing: 2
                                    lineHeight: 20
                                    lineHeightMode: Text.FixedHeight
                                }
                            }
                        }
                    }
                }
            }
            Faces {
                id: faces
            }
        }
    }
}
