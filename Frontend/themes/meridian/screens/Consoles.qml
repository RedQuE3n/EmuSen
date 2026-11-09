pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Effects
import EmuSen.Theme
import "../parts"

// Meridian's Choose a console: the rail of the library's shelves, then each maker's heading and its consoles, scrolling in units under faded edges; the band shows the focused console's body, words and three covers whole (meridian.md §3).
ThemeScreen {
    id: screen
    objectName: "consolesScreen"
    role: ThemeScreen.Consoles
    focusRole: ThemeScreen.Navigation
    // An entry of the rail, a shelf or a console, at its rank: shelves 0 to 2, then the consoles in maker order.
    component Entry: RailEntry {
        id: entry
        property int rank
        property string key
        property bool isConsole
        property string fullName
        property string facts
        property string engineName
        property string engineMark
        property bool libretro
        property int groupStart
        property int groupSize
        property int groupIndex
        property int count
        // Where the entry lies in the rail's list, its maker's heading kept with a group's first.
        property real spanTop
        readonly property bool chosen: root.railRank === entry.rank
        // Rows wholly outside the viewport go unpainted, by opacity so they can still take the focus.
        opacity: entry.y + entry.height > rail.contentY && entry.y < rail.contentY + rail.height ? 1 : 0
        controller: frame.controller
        faces: root.faceSet
        held: false
        onChosenChanged: if (entry.chosen && !entry.activeFocus) entry.forceActiveFocus()
        onActiveFocusChanged: {
            if (!entry.activeFocus)
                return
            root.railRank = entry.rank
            root.focused = entry
            Actions.focus(entry.key)
            rail.contentY = Math.max(0, Math.min(Math.max(rail.contentY, entry.y + entry.height - rail.height), entry.spanTop, Math.max(0, rail.contentHeight - rail.height)))
        }
    }
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
            // The faces for the inline components, whose own property of that name would hide the id.
            readonly property Faces faceSet: faces
            readonly property real bandX: 376
            readonly property real bandWidth: root.width - 440
            readonly property bool light: Tokens.variant === Tokens.Light
            // The rail's focused rank, which Up, Down and the shoulders move; the entry of that rank takes the focus.
            property int railRank: -1
            readonly property int lastRank: 2 + Library.consoles.count
            // A shoulder's jump back lands on the last of the maker before, then climbs to that maker's first entry.
            property bool toGroupHead: false
            property Entry focused: null
            readonly property bool consoleShown: root.focused !== null && root.focused.isConsole
            readonly property int allGames: (shelfEntries.count > 0 ? shelfEntries.itemAt(0) as Entry : null)?.count ?? 0
            Backdrop {
                objectName: "consolesSky"
                anchors.fill: parent
                glowHue: root.consoleShown ? root.focused.hue : -1
                glowX: (root.width - 168) / Math.max(1, root.width)
                glowY: 176 / 800
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
                        text: Words.summary(Library.consoles.count, root.allGames)
                        color: Tokens.muted
                        font.family: faces.regular
                        font.pixelSize: 20
                        lineHeight: 24
                        lineHeightMode: Text.FixedHeight
                    }
                    Glass {
                        objectName: "rail"
                        x: 64
                        y: 96
                        width: 280
                        height: 630
                    }
                    Flickable {
                        id: rail
                        objectName: "railList"
                        x: 64
                        y: 112
                        width: 280
                        height: 598
                        clip: true
                        contentWidth: 280
                        boundsBehavior: Flickable.StopAtBounds
                        contentHeight: list.childrenRect.height
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
                        Item {
                            id: list
                            objectName: "railContent"
                            width: 280
                            Keys.onUpPressed: root.railRank = Math.max(0, root.railRank - 1)
                            Keys.onDownPressed: root.railRank = Math.min(root.lastRank, root.railRank + 1)
                            Keys.onRightPressed: (event) => event.accepted = true
                            Keys.onPressed: (event) => {
                                if (event.key === Qt.Key_PageDown && root.focused) {
                                    root.railRank = root.focused.isConsole ? Math.min(root.lastRank, 3 + root.focused.groupStart + root.focused.groupSize) : Math.min(root.lastRank, 3)
                                    event.accepted = true
                                } else if (event.key === Qt.Key_PageUp && root.focused && root.focused.isConsole && root.focused.groupStart > 0) {
                                    root.toGroupHead = true
                                    root.railRank = 3 + root.focused.groupStart - 1
                                    event.accepted = true
                                }
                            }
                            Repeater {
                                id: shelfEntries
                                model: Library.shelves
                                Entry {
                                    id: shelf
                                    required property int index
                                    required key
                                    required name
                                    required line
                                    required count
                                    objectName: "entry.shelf." + shelf.index
                                    rank: shelf.index
                                    focus: Browse.section.length > 0 ? shelf.key === Browse.section : shelf.index === 0
                                    x: 8
                                    y: shelf.index * 54
                                    spanTop: shelf.y
                                    mark: Marks.path(shelf.key)
                                    onActivated: Actions.openPlace(shelf.key)
                                }
                            }
                            Repeater {
                                id: consoleEntries
                                model: Library.consoles
                                Entry {
                                    id: consoleEntry
                                    required key
                                    required name
                                    required hue
                                    required property string displayName
                                    required property string maker
                                    required property int year
                                    required count
                                    required property string body
                                    required property string engineKind
                                    required property int group
                                    required engineName
                                    required engineMark
                                    required libretro
                                    required groupStart
                                    required groupIndex
                                    required groupSize
                                    readonly property int place: consoleEntry.groupStart + consoleEntry.groupIndex
                                    objectName: "entry.console." + consoleEntry.place
                                    isConsole: true
                                    rank: 3 + consoleEntry.place
                                    focus: consoleEntry.key === Browse.section
                                    x: 8
                                    y: 162 + (consoleEntry.group + 1) * 40 + consoleEntry.place * 54
                                    spanTop: consoleEntry.groupIndex === 0 ? consoleEntry.y - 40 : consoleEntry.y
                                    world: consoleEntry.body
                                    line: Words.games(consoleEntry.count)
                                    fullName: consoleEntry.displayName
                                    facts: Words.facts(consoleEntry.maker, consoleEntry.year, consoleEntry.count, Words.runsOn(consoleEntry.engineName, consoleEntry.engineKind))
                                    onChosenChanged: if (consoleEntry.chosen && root.toGroupHead) {
                                        root.toGroupHead = false
                                        root.railRank = 3 + consoleEntry.groupStart
                                    }
                                    onActivated: Actions.openConsole(consoleEntry.key)
                                }
                            }
                            // Each maker's heading, before its first console: the maker upper letter-spaced with the group's size, a rule at its foot.
                            Repeater {
                                model: Library.consoles
                                Item {
                                    id: heading
                                    required property string maker
                                    required property int group
                                    required property int groupIndex
                                    required property int groupStart
                                    required property int groupSize
                                    objectName: "heading." + heading.group
                                    visible: heading.groupIndex === 0 && heading.y + 40 > rail.contentY && heading.y < rail.contentY + rail.height
                                    x: 0
                                    y: 162 + heading.group * 40 + heading.groupStart * 54
                                    width: 280
                                    height: 40
                                    // Drawn for a group's first console alone, so the other rows make no words.
                                    Loader {
                                        active: heading.groupIndex === 0
                                        sourceComponent: Component {
                                            Item {
                                                width: 280
                                                height: 40
                                                Row {
                                                    x: 16
                                                    y: 12
                                                    spacing: 6
                                                    visible: heading.maker.length > 0
                                                    Text {
                                                        objectName: "headingMaker." + heading.group
                                                        width: Math.min(implicitWidth, 232 - headingCount.implicitWidth - 2 * 6 - headingBetween.implicitWidth)
                                                        text: heading.maker
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
                                                        id: headingBetween
                                                        text: Words.between
                                                        color: Tokens.muted
                                                        font.family: faces.condensed
                                                        font.weight: Font.Medium
                                                        font.pixelSize: 19
                                                        lineHeight: 22.8
                                                        lineHeightMode: Text.FixedHeight
                                                    }
                                                    Text {
                                                        id: headingCount
                                                        text: heading.groupSize
                                                        color: Tokens.muted
                                                        font.family: faces.condensed
                                                        font.weight: Font.Medium
                                                        font.pixelSize: 19
                                                        font.letterSpacing: 2
                                                        lineHeight: 22.8
                                                        lineHeightMode: Text.FixedHeight
                                                    }
                                                }
                                                Rectangle {
                                                    objectName: "headingRule." + heading.group
                                                    x: 16
                                                    y: 39
                                                    width: 232
                                                    height: 1
                                                    color: Tokens.rule
                                                }
                                            }
                                        }
                                    }
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
                        x: 334
                        y: 112
                        width: 2
                        height: 598
                        shown: rail.contentHeight > 0 ? rail.height / rail.contentHeight : 1
                        at: rail.contentHeight > 0 ? rail.contentY / rail.contentHeight : 0
                    }

                    // The band, cross-fading as the rail moves.
                    Item {
                        id: band
                        objectName: "band"
                        anchors.fill: parent
                        NumberAnimation {
                            id: bandFade
                            target: band
                            property: "opacity"
                            from: 0
                            to: 1
                            duration: Tokens.reducedMotion ? 0 : Tokens.fade
                            easing.type: Tokens.fadeCurve
                        }
                        Connections {
                            target: root
                            function onFocusedChanged() { bandFade.restart() }
                        }
                        Body {
                            objectName: "bandWorld"
                            visible: root.consoleShown
                            x: root.width - 248
                            y: 96
                            width: 160
                            height: 160
                            world: root.consoleShown ? root.focused.world : ""
                            hue: root.consoleShown ? root.focused.hue : -1
                        }
                        Scene {
                            visible: root.consoleShown
                            x: root.width - 248
                            y: 96
                            width: 160
                            height: 160
                            Glyph {
                                x: -97
                                y: 16
                                width: 12
                                height: 12
                                path: Marks.sparkle
                                filled: true
                                ink: root.light ? "#8A8FA8" : "#FFFFFF"
                            }
                            Glyph {
                                x: 163
                                y: 70
                                width: 10
                                height: 10
                                path: Marks.sparkle
                                filled: true
                                ink: root.light ? "#8A8FA8" : "#FFFFFF"
                            }
                            Glyph {
                                x: -92
                                y: 139
                                width: 9
                                height: 9
                                path: Marks.sparkle
                                filled: true
                                ink: root.light ? "#8A8FA8" : "#FFFFFF"
                            }
                        }
                        Column {
                            x: root.bandX
                            y: 96
                            width: root.bandWidth - (root.consoleShown ? 208 : 0)
                            spacing: 8
                            Text {
                                objectName: "bandEyebrow"
                                visible: !root.consoleShown
                                text: Words.places
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: 20
                                font.letterSpacing: 2
                                lineHeight: 24
                                lineHeightMode: Text.FixedHeight
                            }
                            Text {
                                objectName: "bandTitle"
                                width: parent.width
                                text: root.focused ? (root.consoleShown ? root.focused.fullName : root.focused.name) : ""
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
                                objectName: "bandFacts"
                                width: parent.width
                                text: root.focused ? (root.consoleShown ? root.focused.facts : root.focused.line) : ""
                                elide: Text.ElideRight
                                color: Tokens.muted
                                font.family: faces.regular
                                font.pixelSize: root.consoleShown ? 22 : 26
                                lineHeight: root.consoleShown ? 26.4 : 31.2
                                lineHeightMode: Text.FixedHeight
                            }
                            Item {
                                width: 1
                                height: 4
                                visible: root.consoleShown
                            }
                            Scene {
                                objectName: "bandEngine"
                                visible: root.consoleShown && root.focused.engineName.length > 0
                                width: engineRow.width
                                height: 22
                                Row {
                                    id: engineRow
                                    spacing: 6
                                    readonly property real hue: root.consoleShown ? root.focused.hue : 0
                                    readonly property bool libretro: root.consoleShown && root.focused.libretro
                                    readonly property color ink: engineRow.libretro ? (root.light ? Tokens.muted : "#C9C9D0") : (root.light ? Qt.hsla(engineRow.hue / 360, 0.6, 0.3, 1) : Qt.hsla(engineRow.hue / 360, 0.85, 0.8, 1))
                                    EngineMark {
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: 15
                                        height: 15
                                        mark: root.consoleShown ? root.focused.engineMark : ""
                                        libretro: engineRow.libretro
                                        ink: engineRow.ink
                                    }
                                    Text {
                                        objectName: "bandEngineName"
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: Math.min(implicitWidth, 150)
                                        text: root.consoleShown ? root.focused.engineName : ""
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
                                        height: 22
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
                        }
                        // Three covers whole, 160 tall and at most 224 wide, centred in the band with their feet at 660.
                        Row {
                            id: coverRow
                            objectName: "coverRow"
                            x: root.bandX + (root.bandWidth - coverRow.width) / 2
                            y: 500
                            height: 160
                            spacing: 24
                            Repeater {
                                id: covers
                                // The focused console's or shelf's first three, recent first.
                                model: root.focused ? Library.firstOf(root.focused.key, 3) : null
                                CoverTile {
                                    id: coverTile
                                    required property int index
                                    required title
                                    required property string cover
                                    required property real coverShape
                                    required property real consoleHue
                                    required favourite
                                    objectName: "bandCover." + coverTile.index
                                    visible: coverTile.index < 3
                                    width: Math.min(224, Math.round(160 * (coverTile.coverShape > 0 ? coverTile.coverShape : 0.75)))
                                    height: 160
                                    source: coverTile.cover
                                    hue: coverTile.consoleHue
                                    mark: Marks.heart
                                    lit: false
                                    pixelScale: root.pixelScale
                                    RectangularShadow {
                                        z: -1
                                        x: coverTile.paintedX
                                        y: coverTile.paintedY
                                        width: coverTile.paintedWidth
                                        height: coverTile.paintedHeight
                                        offset.y: 8
                                        blur: 24
                                        radius: Tokens.tileRadius
                                        color: "#80000000"
                                    }
                                }
                            }
                        }
                        Column {
                            x: root.bandX
                            y: 440
                            spacing: 8
                            visible: root.focused !== null && covers.count === 0
                            Text {
                                objectName: "bandNoGames"
                                text: Words.noGames
                                color: Tokens.textStrong
                                font.family: faces.condensed
                                font.weight: Font.Medium
                                font.pixelSize: 32
                                lineHeight: 38.4
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
                    }
                }
            }
        }
    }
}
