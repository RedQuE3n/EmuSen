pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme
import "../parts"

// Eclipse's Settings as its boards draw it (EclipseSettings.dc.html, gen_settings.py): the limb and the page's glow, the header, the sections, the page's heading and rows, the About panel and the help line, every measure the board's in canvas units (EmuSen_Qt.md §3.4.11).
ThemeScreen {
    id: screen
    objectName: "settingsScreen"
    role: ThemeScreen.Settings
    focusItems: [
        root.isConsoles ? root.consolesScope : root.rowsScope
    ]
    NormalizedCanvas {
        id: canvas
        objectName: "settingsCanvas"
        FocusScope {
            id: root
            objectName: "settings"
            anchors.fill: parent
            focus: true
            readonly property Item rowsScope: rowsScope
            readonly property Item consolesScope: consolesScope
            readonly property bool isConsoles: Settings.consolesSubtitle.length > 0
            // The window's right edge in board pixels: the boards' 1920 at their shape, wider at a wider window.
            readonly property real wide: root.width / design.unit
            // The foot of the board's ninth row, where the rows' window ends (G:67-69: 150 + 56 + 12 + 9 × 84 + 8 × 10).
            readonly property real rowsFoot: 1064
            // The help line's entries as the boards name them, X drawn declined on a row with no default (1.11, 1.15).
            readonly property var help: Settings.help >= 0 ? Help.entries(Settings.help, { settings: true, consoles: root.isConsoles, cards: Settings.cardsHelp, filtered: Settings.consolesFiltered, filterable: Settings.hasConsolesWithGames, defaultable: Settings.defaultable }) : null
            Board {
                id: design
                designHeight: canvas.designHeight
            }
            Ink {
                id: ink
            }
            SettingsInk {
                id: own
            }
            // The ground, the page's glow 620 px short of the board's right edge, and the limb's glow (G:88-91): blurred fields, the page's colour the section's own.
            Backdrop {
                objectName: "settingsBackdrop"
                anchors.fill: parent
                ground: Tokens.surface
                pixelScale: canvas.unit
                fields: [
                    BackdropField { x: (root.wide - 620) * design.unit; y: 60 * design.unit; width: 900 * design.unit; height: 900 * design.unit; color: own.pageGlow; blur: 150 * design.unit; opacity: 0.4 },
                    BackdropField { x: (-1560 - 6) * design.unit; y: (-700 - 6) * design.unit; width: (2000 + 12) * design.unit; height: (2600 + 12) * design.unit; color: own.limbGlow; blur: 20 * design.unit; opacity: own.limbGlowOpacity }
                ]
            }
            // The limb: its 2 px rim, then the dark ellipse (G:91).
            Limb {
                objectName: "limbRim"
                x: (-1560 - 2) * design.unit
                y: (-700 - 2) * design.unit
                width: (2000 + 4) * design.unit
                height: (2600 + 4) * design.unit
                fill: own.limbRim
            }
            Limb {
                objectName: "limb"
                x: -1560 * design.unit
                y: -700 * design.unit
                width: 2000 * design.unit
                height: 2600 * design.unit
                fill: own.limb
            }
            Frame {
                id: frame
                anchors.fill: parent
                surfaceDrawn: false
                helpShown: false
                // The help's entries are the frame's actions, which the pad's buttons invoke; the help line draws them.
                help: root.help
                padFamily: Device.family
                focus: true
                controller.navigationShown: false
                Connections {
                    target: frame.controller
                    function onActionInvoked(key) { Actions.act(key) }
                    function onBackDeclined() { Actions.settingsBack() }
                }
                Item {
                    x: -frame.safeX
                    y: -frame.safeY
                    width: root.width
                    height: root.height
                    Brand {
                        id: brand
                        objectName: "settingsBrand"
                        unit: design.unit
                        named: false
                        x: 96 * design.unit
                        y: 80 * design.unit - height / 2
                    }
                    Line {
                        objectName: "settingsTitle"
                        unit: design.unit
                        x: brand.x + brand.width + 20 * design.unit
                        y: 80 * design.unit - height / 2
                        text: Settings.title
                        color: Tokens.text
                        size: 34
                        weight: Tokens.weightTitle
                    }
                    Status {
                        objectName: "settingsStatus"
                        unit: design.unit
                        controller: frame.controller
                        searchable: false
                        glass: false
                        battery: true
                        // No glass, so no inset: the clock ends at the margin (G:93, 98).
                        pad: 0
                        x: parent.width - 96 * design.unit - width
                        y: 80 * design.unit - height / 2
                    }
                    SettingsNav {
                        unit: design.unit
                        x: 72 * design.unit
                        y: 170 * design.unit
                    }
                    SettingsConsoles {
                        id: consolesScope
                        unit: design.unit
                        controller: frame.controller
                        pixelScale: canvas.unit
                        visible: root.isConsoles
                        Binding {
                            target: consolesScope
                            property: "focus"
                            value: root.isConsoles
                        }
                    }
                    // A page's trail over its heading, with inline kind pills on Themes (G:66, R-20261123-39, EclipseSettingsThemes:40).
                    Row {
                        id: crumbRow
                        objectName: "settingsCrumbRow"
                        visible: !root.isConsoles && crumb.text.length > 0
                        x: 500 * design.unit
                        y: 150 * design.unit
                        height: Settings.inlinePills.length > 0 ? 34 * design.unit : crumb.height
                        spacing: 18 * design.unit
                        Line {
                            id: crumb
                            objectName: "settingsCrumb"
                            unit: design.unit
                            boardLine: true
                            y: (crumbRow.height - height) / 2
                            text: Settings.boardTrailLine
                            color: Tokens.muted
                            size: 20
                        }
                        Row {
                            id: inlinePills
                            objectName: "settingsInlinePills"
                            visible: Settings.inlinePills.length > 0
                            spacing: 8 * design.unit
                            Repeater {
                                model: Settings.inlinePills
                                delegate: Rectangle {
                                    id: pill
                                    required property int index
                                    required property string modelData
                                    readonly property bool active: pill.index === Settings.inlinePill
                                    objectName: "inlinePill." + pill.index
                                    height: 34 * design.unit
                                    width: pillLabel.implicitWidth + 32 * design.unit
                                    radius: 17 * design.unit
                                    color: pill.active ? Tokens.accent : "transparent"
                                    border.width: pill.active ? 0 : 2 * design.unit
                                    border.color: own.ringMuted
                                    Line {
                                        id: pillLabel
                                        objectName: "inlinePillWords." + pill.index
                                        unit: design.unit
                                        boardLine: true
                                        anchors.centerIn: parent
                                        text: pill.modelData
                                        size: 20
                                        weight: pill.active ? Tokens.weightTitle : Font.Normal
                                        color: pill.active ? Tokens.overAccent : Tokens.text
                                    }
                                    MouseArea {
                                        anchors.fill: parent
                                        onClicked: Actions.settingsInlinePill(pill.index)
                                    }
                                }
                            }
                        }
                    }
                    Line {
                        id: heading
                        objectName: "settingsHeading"
                        unit: design.unit
                        visible: !root.isConsoles
                        x: 500 * design.unit
                        // Under the trail by its 6 px margin and the column's 10 px gap (G:66-68).
                        y: crumbRow.visible ? crumbRow.y + crumbRow.height + (6 + 10) * design.unit : 150 * design.unit
                        // Its box the board's line box, its words on the board's baseline, as the rows' are (G:68).
                        boardLine: true
                        text: Settings.boardHeading
                        color: Tokens.text
                        size: 44
                        weight: Tokens.weightTitle
                    }
                    // The rows, from 22 px under the heading to the board's ninth row's foot, the focused one kept in the window.
                    FocusScope {
                        id: rowsScope
                        objectName: "settingsRows"
                        visible: !root.isConsoles
                        x: 500 * design.unit
                        y: heading.y + heading.height + (12 + 10) * design.unit
                        width: 820 * design.unit
                        height: root.rowsFoot * design.unit - y
                        focus: true
                        Binding {
                            target: rowsScope
                            property: "focus"
                            value: !root.isConsoles && (Settings.chipCount === 0 || Settings.focused < Settings.count)
                        }
                        // The rows' grammar first; on a page under a section the shoulders are the page's, the next console, as the program's are.
                        Keys.onPressed: (event) => event.accepted = (event.key === Qt.Key_Left && Actions.settingsStep(Settings.focused, -1)) || (event.key === Qt.Key_Right && Actions.settingsStep(Settings.focused, 1)) || walk.key(event.key, event.modifiers) || (event.key === Qt.Key_PageUp && Actions.settingsShoulder(-1)) || (event.key === Qt.Key_PageDown && Actions.settingsShoulder(1))
                        MeasuredWalk {
                            id: walk
                            objectName: "settingsWalk"
                            // The model's focus is the one that stands: the walk follows it, and tells only the moves it makes itself.
                            follows: Settings.focused
                            model: root.isConsoles ? null : Settings.rows
                            window: rowsScope.height
                            gap: 10 * design.unit
                            sections: Settings.sections.length
                            section: Settings.section
                            trailShown: Settings.trail.length > 0
                            onStepped: (row, by) => Actions.settingsStep(row, by)
                            onSlid: (row, amount) => Actions.settingsSlide(row, amount)
                            onChosen: (row) => Actions.settingsChoose(row)
                            onSectionChosen: (section) => Actions.settingsSection(section)
                            onMoved: (row) => Actions.settingsFocus(row)
                        }
                        // The program's rows changed: the walk reads them again, on the model's focus.
                        Connections {
                            target: Settings
                            function onShown() { walk.read() }
                        }
                        WheelHandler {
                            onWheel: (event) => walk.scrollTo(walk.first + (event.angleDelta.y < 0 ? 1 : -1))
                        }
                        Column {
                            id: list
                            y: -walk.offset
                            width: parent.width
                            spacing: walk.gap
                            Repeater {
                                id: rows
                                model: root.isConsoles ? null : Settings.rows
                                onItemAdded: (index, item) => walk.measure(index, item.height)
                                // A row's place in the column, always laid out, so the row a window has scrolled past keeps its place while it is not drawn.
                                delegate: FocusScope {
                                    id: slot
                                    required property int index
                                    required property string boardLabel
                                    required property string boardAbout
                                    required property string boardValue
                                    required property int kind
                                    required property bool isSwitch
                                    required property bool switchOn
                                    required property bool code
                                    required property int boardForm
                                    width: list.width
                                    height: row.height
                                    Binding {
                                        target: slot
                                        property: "focus"
                                        value: slot.index === Settings.focused
                                    }
                                    onHeightChanged: walk.measure(slot.index, slot.height)
                                    SettingsRow {
                                        id: row
                                        unit: design.unit
                                        controller: frame.controller
                                        pixelScale: canvas.unit
                                        index: slot.index
                                        label: slot.boardLabel
                                        about: slot.boardAbout
                                        value: slot.boardValue
                                        kind: slot.kind
                                        isSwitch: slot.isSwitch
                                        switchOn: slot.switchOn
                                        code: slot.code
                                        form: slot.boardForm
                                        wrapAbout: Settings.boardWrapAbout
                                        width: slot.width
                                        focus: true
                                        // A row wholly in the window is drawn; one the window has scrolled past is not.
                                        visible: slot.y - walk.offset > -0.5 && slot.y + slot.height - walk.offset < rowsScope.height + 0.5
                                        onActiveFocusChanged: {
                                            if (row.activeFocus)
                                                Actions.settingsFocus(slot.index)
                                        }
                                        onActivated: walk.key(Qt.Key_Return, 0)
                                        // A chevron is Left or Right on its row: a stepped value steps, a slider slides.
                                        onStepped: (by) => {
                                            walk.focusRow(slot.index)
                                            Actions.settingsStep(slot.index, by)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    SettingsAside {
                        id: aside
                        unit: design.unit
                        controller: frame.controller
                        pixelScale: canvas.unit
                        visible: !root.isConsoles
                        heading: Settings.boardAboutHeading
                        x: parent.width - 96 * design.unit - width
                        y: 236 * design.unit
                        Binding {
                            target: aside
                            property: "focus"
                            value: !root.isConsoles && Settings.chipCount > 0 && Settings.focused >= Settings.count
                        }
                    }
                    HelpLine {
                        objectName: "settingsHelp"
                        unit: design.unit
                        controller: frame.controller
                        model: root.help
                        ink: ink.help
                        keyPairs: true
                        boxMenu: Settings.boxMenu
                        // The boards' foot: its row 40 px over the bottom, 38 px tall for a ringed glyph (G:114).
                        middle: 1141
                        Line {
                            objectName: "settingsPlace"
                            unit: design.unit
                            visible: Settings.total > 0
                            text: Words.place(Settings.focused, Settings.total)
                            color: Tokens.muted
                            size: 21
                        }
                    }
                }
            }
        }
    }
}
