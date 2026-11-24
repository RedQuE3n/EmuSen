pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Scraping's eleven media-kind toggle chips in a wrapped 12 px flow, reached by Right from the switch rows (EclipseSettingsScraping:69; R-20261116-22; EmuSen_Qt.md §3.4.11).
FocusScope {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property var controller
    // Scene pixels to a canvas unit, so a focused chip's glow blurs at the pixels drawn.
    property real pixelScale: 1
    objectName: "asideChips"
    width: parent ? parent.width : 0
    height: flow.implicitHeight
    Ink {
        id: ink
    }
    Flow {
        id: flow
        width: root.width
        spacing: 12 * root.unit
        Repeater {
            id: chips
            model: Settings.chips
            delegate: Walkable {
                id: chip
                required property int index
                required property string key
                required property string label
                required property bool switchOn
                required property int row
                readonly property real radius: 27 * root.unit
                objectName: "chip." + chip.index
                controller: root.controller
                width: content.width + 40 * root.unit
                height: 54 * root.unit
                Accessible.role: Accessible.CheckBox
                Accessible.name: chip.label
                Accessible.checked: chip.switchOn
                Binding {
                    target: chip
                    property: "focus"
                    value: chip.row === Settings.focused
                }
                onActiveFocusChanged: {
                    if (chip.activeFocus)
                        Actions.settingsFocus(chip.row)
                }
                onActivated: Actions.settingsChoose(chip.row)
                Keys.onLeftPressed: (event) => event.accepted = chip.x > root.unit ? Actions.settingsFocus(chip.row - 1) : Actions.settingsBack()
                Keys.onRightPressed: (event) => event.accepted = chip.index + 1 < chips.count && Math.abs((chips.itemAt(chip.index + 1)?.y ?? -1) - chip.y) < root.unit ? Actions.settingsFocus(chip.row + 1) : false
                Keys.onUpPressed: (event) => event.accepted = chip.index >= 2 ? Actions.settingsFocus(chip.row - 2) : false
                Keys.onDownPressed: (event) => event.accepted = chip.index + 2 < chips.count ? Actions.settingsFocus(chip.row + 2) : (chip.index === chips.count - 2 ? Actions.settingsFocus(chip.row + 1) : false)
                Rectangle {
                    objectName: "chipFill." + chip.index
                    anchors.fill: parent
                    radius: chip.radius
                    color: chip.switchOn ? Tokens.accent : "transparent"
                    border.width: 2 * root.unit
                    border.color: chip.switchOn ? Tokens.accent : ink.pillLine
                }
                FocusRing {
                    objectName: "chipRing"
                    anchors.fill: parent
                    visible: chip.activeFocus
                    radius: chip.radius
                    gap: 3 * root.unit
                    thickness: 3 * root.unit
                    glowColor: Tokens.glowColor
                    glowBlur: 40 * root.unit
                    glowSpread: 8 * root.unit
                    bandRadius: 33 * root.unit
                    pixelScale: root.pixelScale
                }
                Row {
                    id: content
                    x: 20 * root.unit
                    y: (chip.height - height) / 2
                    spacing: 10 * root.unit
                    Image {
                        objectName: "chipCheck." + chip.index
                        visible: chip.switchOn
                        y: (content.height - height) / 2
                        width: 18 * root.unit
                        height: width
                        source: "../assets/check.svg"
                        sourceSize: Qt.size(72, 72)
                        smooth: true
                    }
                    Rectangle {
                        objectName: "chipDot." + chip.index
                        visible: !chip.switchOn
                        y: (content.height - height) / 2
                        width: 14 * root.unit
                        height: width
                        radius: width / 2
                        color: "transparent"
                        border.width: 2 * root.unit
                        border.color: Tokens.muted
                    }
                    Line {
                        objectName: "chipWords." + chip.index
                        unit: root.unit
                        boardLine: true
                        y: (content.height - height) / 2
                        text: chip.label
                        color: chip.switchOn ? Tokens.overAccent : Tokens.muted
                        size: 19
                        weight: Tokens.weightTitle
                    }
                }
            }
        }
    }
}
