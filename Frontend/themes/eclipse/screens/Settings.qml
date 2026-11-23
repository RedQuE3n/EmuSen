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
        root.rowsScope
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
            // The window's right edge in board pixels: the boards' 1920 at their shape, wider at a wider window.
            readonly property real wide: root.width / design.unit
            // The foot of the board's ninth row, where the rows' window ends (G:67-69: 150 + 56 + 12 + 9 × 84 + 8 × 10).
            readonly property real rowsFoot: 1064
            // The help line's entries as the boards name them, X drawn declined on a row with no default (1.11).
            readonly property var help: Settings.help >= 0 ? Help.entries(Settings.help, { settings: true, defaultable: Settings.defaultable }) : null
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
                    // A page's trail over its heading, every level above the page (G:66, R-20261123-39).
                    Line {
                        id: crumb
                        objectName: "settingsCrumb"
                        unit: design.unit
                        boardLine: true
                        visible: text.length > 0
                        x: 500 * design.unit
                        y: 150 * design.unit
                        text: Settings.trailLine
                        color: Tokens.muted
                        size: 20
                    }
                    LineBox {
                        id: headingBox
                        size: 44
                        weight: Tokens.weightTitle
                    }
                    Line {
                        id: heading
                        objectName: "settingsHeading"
                        unit: design.unit
                        x: 500 * design.unit
                        // Under the trail by its 6 px margin and the column's 10 px gap (G:66-68).
                        y: crumb.visible ? crumb.y + crumb.height + (6 + 10) * design.unit : 150 * design.unit
                        // Its box CSS's normal line height, as the board's h2 has it (G:68).
                        height: headingBox.box * design.unit
                        text: Settings.heading
                        color: Tokens.text
                        size: 44
                        weight: Tokens.weightTitle
                    }
                    // The rows, from 22 px under the heading to the board's ninth row's foot, the focused one kept in the window.
                    FocusScope {
                        id: rowsScope
                        objectName: "settingsRows"
                        x: 500 * design.unit
                        y: heading.y + heading.height + (12 + 10) * design.unit
                        width: 820 * design.unit
                        height: root.rowsFoot * design.unit - y
                        focus: true
                        // The rows' grammar first; on a page under a section the shoulders are the page's, the next console, as the program's are.
                        Keys.onPressed: (event) => event.accepted = walk.key(event.key, event.modifiers) || (event.key === Qt.Key_PageUp && Actions.settingsShoulder(-1)) || (event.key === Qt.Key_PageDown && Actions.settingsShoulder(1))
                        MeasuredWalk {
                            id: walk
                            objectName: "settingsWalk"
                            // The model's focus is the one that stands: the walk follows it, and tells only the moves it makes itself.
                            follows: Settings.focused
                            model: Settings.rows
                            window: rowsScope.height
                            gap: 10 * design.unit
                            sections: Settings.sections.length
                            section: Settings.section
                            trailShown: Settings.trail.length > 0
                            onStepped: (row, by) => Actions.settingsStep(row, by)
                            onSlid: (row, amount) => Actions.settingsSlide(row, amount)
                            onChosen: (row) => Actions.settingsChoose(row)
                            onSectionChosen: (section) => Actions.settingsSection(section)
                            onWalked: {
                                if (walk.current >= 0 && walk.current !== Settings.focused)
                                    Actions.settingsFocus(walk.current)
                            }
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
                                model: Settings.rows
                                onItemAdded: (index, item) => walk.measure(index, item.height)
                                // A row's place in the column, always laid out, so the row a window has scrolled past keeps its place while it is not drawn.
                                delegate: FocusScope {
                                    id: slot
                                    required property int index
                                    required property string label
                                    required property string about
                                    required property string value
                                    required property int kind
                                    required property bool isSwitch
                                    required property bool switchOn
                                    required property bool code
                                    required property int form
                                    width: list.width
                                    height: row.height
                                    focus: slot.index === Settings.focused
                                    onHeightChanged: walk.measure(slot.index, slot.height)
                                    SettingsRow {
                                        id: row
                                        unit: design.unit
                                        controller: frame.controller
                                        pixelScale: canvas.unit
                                        index: slot.index
                                        label: slot.label
                                        about: slot.about
                                        value: slot.value
                                        kind: slot.kind
                                        isSwitch: slot.isSwitch
                                        switchOn: slot.switchOn
                                        code: slot.code
                                        form: slot.form
                                        width: slot.width
                                        focus: true
                                        // A row wholly in the window is drawn; one the window has scrolled past is not.
                                        visible: slot.y - walk.offset > -0.5 && slot.y + slot.height - walk.offset < rowsScope.height + 0.5
                                        onActiveFocusChanged: {
                                            if (row.activeFocus && Settings.focused !== slot.index)
                                                Actions.settingsFocus(slot.index)
                                        }
                                        onActivated: walk.key(Qt.Key_Return, 0)
                                        // A chevron is Left or Right on its row: a stepped value steps, a slider slides.
                                        onStepped: (by) => {
                                            walk.focusRow(slot.index)
                                            walk.key(by > 0 ? Qt.Key_Right : Qt.Key_Left, 0)
                                        }
                                    }
                                }
                            }
                        }
                    }
                    SettingsAside {
                        unit: design.unit
                        pixelScale: canvas.unit
                        heading: Settings.aboutHeading
                        x: parent.width - 96 * design.unit - width
                        y: 236 * design.unit
                    }
                    HelpLine {
                        objectName: "settingsHelp"
                        unit: design.unit
                        controller: frame.controller
                        model: root.help
                        ink: ink.help
                        keyPairs: true
                        // The boards' foot: its row 40 px over the bottom, 38 px tall for a ringed glyph (G:114).
                        middle: 1141
                        Line {
                            objectName: "settingsPlace"
                            unit: design.unit
                            visible: Settings.count > 0
                            text: Words.place(Settings.focused, Settings.count)
                            color: Tokens.muted
                            size: 21
                        }
                    }
                }
            }
        }
    }
}
