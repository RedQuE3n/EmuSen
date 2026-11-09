pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Choose a console, "Wheelwork": the focused maker's wheel at the left with its consoles on the spokes' ends and the focus at the top, the dial of the three shelves and a wheel per maker along the foot, and the key under the moon at the right (orrery.md §3).
ThemeScreen {
    id: screen
    objectName: "consolesScreen"
    role: ThemeScreen.Consoles
    NormalizedCanvas {
        id: canvas
        objectName: "consolesCanvas"
        FocusScope {
            id: root
            // First, so the parts read the faces from their first binding.
            Faces {
                id: faces
            }
            objectName: "consoles"
            anchors.fill: parent
            focus: true
            readonly property real pixelScale: canvas.unit
            readonly property bool light: Tokens.variant === Tokens.Light
            readonly property color lineInk: root.light ? Qt.rgba(74 / 255, 58 / 255, 40 / 255, 0.5) : Qt.rgba(140 / 255, 150 / 255, 220 / 255, 0.32)
            readonly property int count: Library.consoles.count
            readonly property real keyX: 760
            readonly property real keyWidth: root.width - 64 - root.keyX
            // How many delegates the Repeaters have made, so a lookup by index is read again as each is.
            property int made: 0
            // The focused node and what the key shows: 0 a world, 1 a shelf, 2 a maker's stop.
            property var focused: null
            readonly property int keyKind: root.focused ? root.focused.kind : -1
            readonly property var keyWorld: root.keyKind === 0 ? root.focused : (root.keyKind === 2 && root.made > 0 ? worlds.itemAt(layout.firsts[root.focused.group] ?? 0) : null)
            readonly property real focusHue: root.keyKind === 0 ? root.focused.hue : -1
            readonly property var allShelf: root.made > 0 && shelves.count > 0 ? shelves.itemAt(0) : null
            readonly property int allGames: root.allShelf ? root.allShelf.count : 0
            WheelLayout {
                id: layout
                width: root.width
                places: 3
                model: Library.consoles
                role: "group"
                weightRole: "count"
            }
            Connections {
                target: Library.consoles
                function onModelReset() { layout.read() }
            }
            Backdrop {
                objectName: "consolesSky"
                anchors.fill: parent
                glowHue: root.focusHue
                glowX: 400 / root.width
                glowY: 242 / 800
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
                help: Help.entries(Help.Console, { group: true, back: true })
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
                        text: Words.summary(root.count, root.allGames)
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Text {
                        objectName: "noConsoles"
                        visible: root.count === 0
                        x: 64
                        y: 358
                        width: root.keyX - 128
                        horizontalAlignment: Text.AlignHCenter
                        text: Words.noConsoles
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 22
                        lineHeight: 26.4
                        lineHeightMode: Text.FixedHeight
                    }
                    // The wheel's layer, cut at the rim so worlds set behind it.
                    Item {
                        objectName: "wheelLayer"
                        x: 0
                        y: 0
                        width: root.width
                        height: 540
                        clip: true
                        visible: layout.wheel >= 0
                        Scene {
                            objectName: "wheel"
                            x: 400 - 160
                            y: 392 - 160
                            width: 320
                            height: 320
                            opacity: layout.dimmed ? 0.3 : 1
                            Rectangle {
                                x: 10
                                y: 10
                                width: 300
                                height: 300
                                radius: 150
                                color: "transparent"
                                border.width: 1
                                border.color: root.lineInk
                            }
                            // Teeth every 7.5°, 4 long, 7 at the spokes' halves.
                            Repeater {
                                model: 48
                                Rectangle {
                                    id: tooth
                                    required property int index
                                    readonly property bool long: tooth.index % 6 === 3
                                    x: 159.5
                                    y: 160 - 150 - (tooth.long ? 7 : 4)
                                    width: 1
                                    height: tooth.long ? 8 : 5
                                    color: root.lineInk
                                    transform: Rotation {
                                        origin.x: 0.5
                                        origin.y: 150 + (tooth.long ? 7 : 4)
                                        angle: tooth.index * 7.5
                                    }
                                }
                            }
                            Repeater {
                                model: 8
                                Rectangle {
                                    id: spoke
                                    required property int index
                                    x: 159.5
                                    y: 10
                                    width: 1
                                    height: 150
                                    color: root.lineInk
                                    opacity: spoke.index === 0 && !layout.dimmed ? 1 : 0.5
                                    transform: Rotation {
                                        origin.x: 0.5
                                        origin.y: 150
                                        angle: spoke.index * layout.step
                                    }
                                }
                            }
                            Rectangle {
                                x: 148
                                y: 148
                                width: 24
                                height: 24
                                radius: 12
                                color: root.light ? Qt.rgba(1, 1, 1, 0.5) : Qt.rgba(38 / 255, 38 / 255, 43 / 255, 0.5)
                                border.width: 1
                                border.color: root.lineInk
                            }
                        }
                        // The worlds within reach of the top hidden, three dots below the hub say so.
                        Row {
                            objectName: "moreDots"
                            visible: layout.hidden > 0
                            x: 390
                            y: 526
                            spacing: 4
                            opacity: layout.dimmed ? 0.3 : 1
                            Repeater {
                                model: 3
                                Rectangle {
                                    width: 4
                                    height: 4
                                    radius: 2
                                    color: Tokens.muted
                                }
                            }
                        }
                        Repeater {
                            id: worlds
                            model: Library.consoles
                            onItemAdded: root.made = worlds.count + shelves.count + makers.count
                            WheelNode {
                                id: world
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
                                required property int group
                                required property int groupIndex
                                required property int groupSize
                                readonly property int kind: 0
                                readonly property int turn: layout.turns[world.index] ?? 0
                                readonly property bool onWheel: layout.onWheel[world.index] ?? false
                                readonly property bool shown: layout.shown[world.index] ?? false
                                readonly property int slotAt: (world.turn + 8) % 8
                                readonly property real size: world.activeFocus ? 96 : 72
                                property real angle: 90 - layout.step * Math.max(-4, Math.min(4, world.turn))
                                Behavior on angle {
                                    NumberAnimation {
                                        duration: Tokens.reel
                                        easing.type: Tokens.reelCurve
                                    }
                                }
                                readonly property real cx: 400 + 150 * Math.cos(world.angle * Math.PI / 180)
                                readonly property real cy: 392 - 150 * Math.sin(world.angle * Math.PI / 180)
                                objectName: "world." + world.index
                                controller: frame.controller
                                layout: layout
                                worlds: worlds
                                shelves: shelves
                                makers: makers
                                made: root.made
                                nodeIndex: world.index
                                Accessible.name: world.name
                                focus: Browse.section.length > 0 ? world.key === Browse.section : world.index === 0
                                opacity: world.shown ? (layout.dimmed ? 0.3 : 1) : 0
                                Behavior on opacity {
                                    NumberAnimation {
                                        duration: Tokens.reel
                                        easing.type: Tokens.reelCurve
                                    }
                                }
                                x: world.cx - world.size / 2
                                // Off the wheel or past its reach, below the rim, where the layer's clip hides it and no pointer finds it.
                                y: world.cy - world.size / 2 + (world.shown ? 0 : 240)
                                width: world.size
                                height: world.size
                                Behavior on width {
                                    NumberAnimation {
                                        duration: Tokens.quick
                                        easing.type: Tokens.quickCurve
                                    }
                                }
                                onActivated: Actions.openConsole(world.key)
                                onActiveFocusChanged: if (world.activeFocus) root.focused = world
                                Body {
                                    objectName: "worldBody." + world.index
                                    anchors.fill: parent
                                    world: world.body
                                    hue: world.hue
                                    focused: world.activeFocus
                                }
                                Rectangle {
                                    objectName: "worldRing." + world.index
                                    visible: world.activeFocus
                                    anchors.centerIn: parent
                                    width: 110
                                    height: 110
                                    radius: 55
                                    color: "transparent"
                                    border.width: Tokens.ringThickness
                                    border.color: Tokens.accent
                                }
                                Scene {
                                    visible: world.activeFocus
                                    anchors.centerIn: parent
                                    width: 0
                                    height: 0
                                    Glyph {
                                        x: -66
                                        y: -42
                                        width: 13
                                        height: 13
                                        path: Marks.sparkle
                                        filled: true
                                        ink: root.light ? "#8A7A66" : "#FFFFFF"
                                    }
                                    Glyph {
                                        x: 50
                                        y: -9
                                        width: 10
                                        height: 10
                                        path: Marks.sparkle
                                        filled: true
                                        ink: root.light ? "#8A7A66" : "#FFFFFF"
                                    }
                                    Glyph {
                                        x: -60
                                        y: 34
                                        width: 7
                                        height: 7
                                        path: Marks.sparkle
                                        filled: true
                                        ink: root.light ? "#8A7A66" : "#FFFFFF"
                                    }
                                }
                                // The world's name and count: above the top slot, right of slots 1 to 3, left of slots 5 to 7.
                                Column {
                                    objectName: "worldLabel." + world.index
                                    visible: world.shown && !layout.dimmed
                                    x: world.slotAt === 0 ? world.size / 2 - 100 : (world.slotAt <= 3 ? world.size / 2 + 44 : world.size / 2 - 44 - 160)
                                    y: world.slotAt === 0 ? world.size / 2 - 108 : world.size / 2 - 22
                                    width: world.slotAt === 0 ? 200 : 160
                                    Text {
                                        objectName: "worldName." + world.index
                                        x: world.slotAt === 0 ? (parent.width - width) / 2 : (world.slotAt <= 3 ? 0 : parent.width - width)
                                        width: Math.min(implicitWidth, 150)
                                        text: world.name
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
                                    Row {
                                        x: world.slotAt === 0 ? (parent.width - width) / 2 : (world.slotAt <= 3 ? 0 : parent.width - width)
                                        spacing: 6
                                        Text {
                                            objectName: "worldCount." + world.index
                                            text: Words.games(world.count)
                                            color: Tokens.muted
                                            font.family: faces.regular
                                            font.pixelSize: 19
                                            lineHeight: 22.8
                                            lineHeightMode: Text.FixedHeight
                                        }
                                        Scene {
                                            anchors.verticalCenter: parent.verticalCenter
                                            width: 13
                                            height: 13
                                            EngineMark {
                                                anchors.fill: parent
                                                mark: world.engineMark
                                                libretro: world.libretro
                                                ink: world.libretro ? (root.light ? Tokens.muted : "#A9A9B2") : Qt.hsla(world.hue / 360, root.light ? 0.6 : 0.5, root.light ? 0.3 : 0.72, 1)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // The maker's name and size under the hub.
                        Row {
                            objectName: "wheelMaker"
                            readonly property var first: root.made > 0 && layout.wheel >= 0 ? worlds.itemAt(layout.firsts[layout.wheel] ?? 0) : null
                            // Hidden while a shelf dims the wheel: words at 30% would read under the contrast floor.
                            visible: makerRow.first !== null && makerRow.first.maker.length > 0 && !layout.dimmed
                            id: makerRow
                            x: 400 - makerRow.width / 2
                            y: 416
                            spacing: 6
                            Text {
                                width: Math.min(implicitWidth, 240)
                                text: makerRow.first ? makerRow.first.maker : ""
                                elide: Text.ElideRight
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
                                text: Words.between
                                color: Tokens.muted
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 19
                                lineHeight: 22.8
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                text: makerRow.first ? makerRow.first.groupSize : 0
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
                    Rectangle {
                        objectName: "rim"
                        x: 64
                        y: 540
                        width: root.width - 128
                        height: 1
                        color: root.lineInk
                    }
                    // The dial's edges fade 40 to transparency where it runs on past them, the sky showing through.
                    EdgeFade {
                        x: 64
                        y: 548
                        width: root.width - 128
                        height: 84
                        fadeLeft: dial.fadeLeft
                        fadeRight: dial.fadeRight
                        length: 40
                        // The dial: the three shelves as glass discs, then a wheel per maker with a tooth per console, under a clamped camera when longer than the safe width.
                        Flickable {
                            id: dial
                            objectName: "dial"
                            anchors.fill: parent
                            clip: true
                            contentWidth: Math.max(dial.width, layout.dialWidth)
                            contentHeight: 84
                            flickableDirection: Flickable.HorizontalFlick
                            boundsBehavior: Flickable.StopAtBounds
                            readonly property bool fadeLeft: dial.contentX > 0.5
                            readonly property bool fadeRight: dial.contentX + dial.width < dial.contentWidth - 0.5
                            Behavior on contentX {
                                NumberAnimation {
                                    duration: Tokens.reel
                                    easing.type: Tokens.reelCurve
                                }
                            }
                            // The camera centres the lit stop; the wheel may move the dial between turns.
                            readonly property real camera: layout.camera
                            onCameraChanged: dial.contentX = dial.camera
                            Repeater {
                                id: shelves
                                model: Library.shelves
                                onItemAdded: root.made = worlds.count + shelves.count + makers.count
                                WheelNode {
                                    id: shelf
                                    required property int index
                                    required property string key
                                    required property string name
                                    required property string line
                                    required property int count
                                    readonly property int kind: 1
                                    readonly property real centre: (layout.dialXs[shelf.index] ?? 0) - 64
                                    objectName: "stop.shelf." + shelf.index
                                    controller: frame.controller
                                    layout: layout
                                    worlds: worlds
                                    shelves: shelves
                                    makers: makers
                                    made: root.made
                                    nodeIndex: root.count + shelf.index
                                    Accessible.name: shelf.name
                                    focus: Browse.section.length > 0 ? shelf.key === Browse.section : root.count === 0 && shelf.index === 0
                                    x: shelf.centre - 68
                                    y: 8
                                    width: 136
                                    height: 76
                                    onActivated: Actions.openPlace(shelf.key)
                                    onActiveFocusChanged: if (shelf.activeFocus) root.focused = shelf
                                    Rectangle {
                                        x: 46
                                        y: 0
                                        width: 44
                                        height: 44
                                        radius: 22
                                        color: root.light ? Qt.rgba(1, 1, 1, 0.5) : Qt.rgba(38 / 255, 38 / 255, 43 / 255, 0.5)
                                        border.width: 1
                                        border.color: root.lineInk
                                        Glyph {
                                            x: 11
                                            y: 11
                                            width: 22
                                            height: 22
                                            path: Marks.path(shelf.key)
                                            ink: shelf.activeFocus ? Tokens.textStrong : Tokens.muted
                                        }
                                    }
                                    Text {
                                        objectName: "stopName.shelf." + shelf.index
                                        x: 0
                                        y: 50
                                        width: 136
                                        horizontalAlignment: Text.AlignHCenter
                                        text: shelf.name
                                        elide: Text.ElideRight
                                        color: shelf.activeFocus ? Tokens.textStrong : Tokens.muted
                                        font.family: faces.condensed
                                        font.weight: Font.Medium
                                        font.pixelSize: 19
                                        font.letterSpacing: 1.2
                                        font.capitalization: Font.AllUppercase
                                        lineHeight: 22.8
                                        lineHeightMode: Text.FixedHeight
                                    }
                                    Rectangle {
                                        visible: shelf.activeFocus
                                        x: 68 - 29
                                        y: -7
                                        width: 58
                                        height: 58
                                        radius: 29
                                        color: "transparent"
                                        border.width: Tokens.ringThickness
                                        border.color: Tokens.accent
                                    }
                                }
                            }
                            Repeater {
                                id: makers
                                model: Library.consoles
                                onItemAdded: root.made = worlds.count + shelves.count + makers.count
                                WheelNode {
                                    id: stop
                                    required property string maker
                                    required property int group
                                    required property int groupIndex
                                    required property int groupSize
                                    readonly property int kind: 2
                                    readonly property bool lit: layout.lit === root.count + 3 + stop.group
                                    readonly property real centre: (layout.dialXs[3 + stop.group] ?? 0) - 64
                                    objectName: "stop.maker." + stop.group
                                    controller: frame.controller
                                    layout: layout
                                    worlds: worlds
                                    shelves: shelves
                                    makers: makers
                                    made: root.made
                                    nodeIndex: root.count + 3 + stop.group
                                    Accessible.name: stop.maker
                                    visible: stop.groupIndex === 0
                                    activeFocusOnTab: stop.groupIndex === 0
                                    x: stop.centre - 68
                                    y: 8
                                    width: 136
                                    height: 76
                                    onActivated: if (stop.upItem) stop.upItem.forceActiveFocus()
                                    onActiveFocusChanged: if (stop.activeFocus) root.focused = stop
                                    DialWheel {
                                        objectName: "makerWheel." + stop.group
                                        x: 46
                                        y: 0
                                        teeth: stop.groupSize
                                        inner: 17
                                        outer: 22
                                        toothWidth: 1.5
                                        ring: false
                                        hub: true
                                        ink: stop.lit ? Tokens.textStrong : Tokens.muted
                                        Rectangle {
                                            anchors.fill: parent
                                            radius: 22
                                            color: "transparent"
                                            border.width: 1
                                            border.color: stop.activeFocus ? Tokens.accent : root.lineInk
                                        }
                                    }
                                    Text {
                                        objectName: "stopName.maker." + stop.group
                                        x: 0
                                        y: 50
                                        width: 136
                                        horizontalAlignment: Text.AlignHCenter
                                        text: stop.maker
                                        elide: Text.ElideRight
                                        color: stop.lit ? Tokens.textStrong : Tokens.muted
                                        font.family: faces.condensed
                                        font.weight: Font.Medium
                                        font.pixelSize: 19
                                        font.letterSpacing: 1.2
                                        font.capitalization: Font.AllUppercase
                                        lineHeight: 22.8
                                        lineHeightMode: Text.FixedHeight
                                    }
                                    Rectangle {
                                        visible: stop.activeFocus
                                        x: 68 - 29
                                        y: -7
                                        width: 58
                                        height: 58
                                        radius: 29
                                        color: "transparent"
                                        border.width: Tokens.ringThickness
                                        border.color: Tokens.accent
                                    }
                                }
                            }
                        }
                    }
                    Track {
                        objectName: "dialTrack"
                        horizontal: true
                        x: 64
                        y: 640
                        width: root.width - 128
                        height: 2
                        shown: layout.dialWidth > 0 ? layout.view / layout.dialWidth : 1
                        at: layout.dialWidth > 0 ? dial.contentX / layout.dialWidth : 0
                    }
                    // The key under the moon: the body, the full name, the facts, the engine line and three covers whole.
                    Moon {
                        objectName: "keyMoon"
                        x: root.width - 64 - 48
                        // Two above the table's 104, so the moon clears the key's name, whose line starts at 150.
                        y: 102
                        width: 48
                        height: 48
                    }
                    Item {
                        id: key
                        objectName: "key"
                        anchors.fill: parent
                        NumberAnimation {
                            id: keyFade
                            target: key
                            property: "opacity"
                            from: 0
                            to: 1
                            duration: Tokens.fade
                            easing.type: Tokens.fadeCurve
                        }
                        Connections {
                            target: root
                            function onFocusedChanged() { keyFade.restart() }
                        }
                        Body {
                            objectName: "keyBody"
                            visible: root.keyWorld !== null
                            x: root.keyX
                            y: 150
                            width: 64
                            height: 64
                            world: root.keyWorld ? root.keyWorld.body : ""
                            hue: root.keyWorld ? root.keyWorld.hue : -1
                            focused: root.keyKind === 0
                        }
                        Rectangle {
                            objectName: "keyDisc"
                            visible: root.keyKind === 1
                            x: root.keyX
                            y: 150
                            width: 64
                            height: 64
                            radius: 32
                            color: root.light ? Qt.rgba(1, 1, 1, 0.5) : Qt.rgba(38 / 255, 38 / 255, 43 / 255, 0.5)
                            border.width: 1
                            border.color: root.lineInk
                            Glyph {
                                x: 16
                                y: 16
                                width: 32
                                height: 32
                                path: root.keyKind === 1 ? Marks.path(root.focused.key) : ""
                                ink: Tokens.textStrong
                            }
                        }
                        Text {
                            objectName: "keyName"
                            x: root.keyX + 80
                            y: 150
                            width: root.keyWidth - 80
                            text: root.keyKind === 0 ? root.focused.displayName : (root.keyKind === 1 ? root.focused.name : (root.keyKind === 2 ? root.focused.maker : ""))
                            wrapMode: Text.Wrap
                            maximumLineCount: 2
                            elide: Text.ElideRight
                            color: Tokens.textStrong
                            font.family: faces.condensed
                            font.weight: Font.Medium
                            font.pixelSize: 44
                            font.letterSpacing: 2
                            font.capitalization: Font.AllUppercase
                            lineHeight: 44
                            lineHeightMode: Text.FixedHeight
                        }
                        Text {
                            objectName: "keyFacts"
                            x: root.keyX
                            y: 272
                            width: root.keyWidth
                            text: root.keyKind === 0 ? Words.facts(root.focused.maker, root.focused.year, root.focused.count, Words.runsOn(root.focused.engineName, root.focused.engineKind)) : (root.keyKind === 1 ? root.focused.line : (root.keyKind === 2 ? Words.summary(root.focused.groupSize, layout.totals[root.focused.group] ?? 0) : ""))
                            wrapMode: Text.Wrap
                            maximumLineCount: 3
                            elide: Text.ElideRight
                            color: Tokens.muted
                            font.family: faces.regular
                            font.pixelSize: 22
                            lineHeight: 26.4
                            lineHeightMode: Text.FixedHeight
                        }
                        Scene {
                            objectName: "keyEngine"
                            visible: root.keyKind === 0 && root.focused.engineName.length > 0
                            x: root.keyX
                            y: 368
                            width: engineRow.width
                            height: 22
                            Row {
                                id: engineRow
                                spacing: 6
                                readonly property real hue: root.keyKind === 0 ? root.focused.hue : 0
                                readonly property bool libretro: root.keyKind === 0 && root.focused.libretro
                                readonly property color ink: engineRow.libretro ? (root.light ? Tokens.muted : "#C9C9D0") : (root.light ? Qt.hsla(engineRow.hue / 360, 0.6, 0.3, 1) : Qt.hsla(engineRow.hue / 360, 0.85, 0.8, 1))
                                EngineMark {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: 15
                                    height: 15
                                    mark: root.keyKind === 0 ? root.focused.engineMark : ""
                                    libretro: engineRow.libretro
                                    ink: engineRow.ink
                                }
                                Text {
                                    objectName: "keyEngineName"
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: Math.min(implicitWidth, 200)
                                    text: root.keyKind === 0 ? root.focused.engineName : ""
                                    elide: Text.ElideRight
                                    color: engineRow.ink
                                    font.family: faces.condensed
                                    font.weight: Font.Medium
                                    font.pixelSize: 19
                                    font.letterSpacing: 1.5
                                    font.capitalization: Font.AllUppercase
                                }
                                Rectangle {
                                    anchors.verticalCenter: parent.verticalCenter
                                    width: tagWord.implicitWidth + 10
                                    height: tagWord.implicitHeight + 2
                                    radius: 4
                                    color: engineRow.libretro ? "transparent" : Qt.hsla(engineRow.hue / 360, 0.6, 0.5, 0.14)
                                    border.color: engineRow.libretro ? "#6E6E78" : Qt.hsla(engineRow.hue / 360, 0.45, root.light ? 0.4 : 0.52, 1)
                                    Text {
                                        id: tagWord
                                        anchors.centerIn: parent
                                        text: engineRow.libretro ? Words.libretroTag : Words.emusenTag
                                        color: engineRow.libretro ? (root.light ? Tokens.muted : "#A9A9B2") : (root.light ? Qt.hsla(engineRow.hue / 360, 0.6, 0.3, 1) : Qt.hsla(engineRow.hue / 360, 0.7, 0.82, 1))
                                        font.family: faces.condensed
                                        font.weight: Font.Medium
                                        font.pixelSize: 19
                                        font.letterSpacing: 0.8
                                    }
                                }
                            }
                        }
                        // Three covers whole in a row, 100 tall and 12 apart, their feet at 520, right-aligned to the key's width.
                        Row {
                            id: coverRow
                            objectName: "keyCovers"
                            x: root.keyX + Math.min(root.keyWidth, 444) - coverRow.width
                            y: 420
                            height: 100
                            spacing: 12
                            layoutDirection: Qt.LeftToRight
                            Repeater {
                                model: root.keyKind === 0 || root.keyKind === 1 ? Library.firstOf(root.focused.key, 3) : null
                                CoverTile {
                                    id: coverTile
                                    required property int index
                                    required title
                                    required property string cover
                                    required property real coverShape
                                    required property real consoleHue
                                    required favourite
                                    objectName: "keyCover." + coverTile.index
                                    visible: coverTile.index < 3
                                    width: Math.min(150, Math.round(100 * (coverTile.coverShape > 0 ? coverTile.coverShape : 0.75)))
                                    height: 100
                                    source: coverTile.cover
                                    hue: coverTile.consoleHue
                                    mark: Marks.heart
                                    lit: false
                                    pixelScale: root.pixelScale
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
