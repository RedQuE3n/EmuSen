pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// Settings Consoles' 2-column grid of 652 by 116 console cards, 6 rows visible, with baseline-aligned subtitle (EclipseSettingsConsoles:39-59; R-20261123-13).
FocusScope {
    id: root
    required property real unit
    required property var controller
    property real pixelScale: 1
    // Six rows of 116 px cards separated by 12 px gaps (EclipseSettingsConsoles:44-46).
    readonly property int visibleRows: 6
    readonly property real cardWidth: 652
    readonly property real cardHeight: 116
    readonly property real colGap: 20
    readonly property real rowGap: 12
    readonly property int totalRows: Math.ceil(Settings.count / 2)
    readonly property int maxFirstRow: Math.max(0, root.totalRows - root.visibleRows)
    readonly property int focusedRow: Math.floor(Math.max(0, Settings.focused) / 2)
    property int firstRow: 0
    objectName: "consolesSection"
    x: 500 * root.unit
    y: 150 * root.unit
    width: 1324 * root.unit
    height: (56 + 22 + root.visibleRows * root.cardHeight + (root.visibleRows - 1) * root.rowGap) * root.unit
    SettingsInk {
        id: own
    }
    Connections {
        target: Settings
        function onShown() { root.firstRow = Math.min(root.maxFirstRow, Math.max(0, root.focusedRow < root.firstRow ? root.focusedRow : (root.focusedRow >= root.firstRow + root.visibleRows ? root.focusedRow - root.visibleRows + 1 : root.firstRow))) }
    }
    Line {
        id: heading
        objectName: "consolesHeading"
        unit: root.unit
        boardLine: true
        x: 0
        y: 0
        text: Settings.boardHeading
        color: Tokens.text
        size: 44
        weight: Tokens.weightTitle
    }
    Line {
        id: subtitle
        objectName: "consolesSubtitle"
        unit: root.unit
        boardLine: true
        x: root.width - width
        y: heading.y + (heading.boardBaseline - subtitle.boardBaseline) * root.unit
        text: Settings.consolesSubtitle
        color: Tokens.muted
        size: 20
    }
    Item {
        id: grid
        objectName: "consolesGrid"
        x: 0
        y: heading.height + (12 + 10) * root.unit
        width: root.width
        height: (root.visibleRows * root.cardHeight + (root.visibleRows - 1) * root.rowGap) * root.unit
        WheelHandler {
            onWheel: (event) => root.firstRow = Math.min(root.maxFirstRow, Math.max(0, root.firstRow + (event.angleDelta.y < 0 ? 1 : -1)))
        }
        Repeater {
            id: cards
            model: Settings.rows
            delegate: Walkable {
                id: card
                required property int index
                required property string boardLabel
                required property int planet
                required property string subtitle
                required property string chip
                required property bool chipAttention
                required property bool runs
                readonly property real u: root ? root.unit : 0
                readonly property int rowOf: Math.floor(card.index / 2)
                readonly property int colOf: card.index % 2
                readonly property bool lit: card.activeFocus
                readonly property real radius: 20 * card.u
                objectName: "console." + card.index
                controller: root.controller
                Accessible.name: card.boardLabel
                Accessible.description: card.subtitle
                x: card.colOf * (root.cardWidth + root.colGap) * card.u
                y: (card.rowOf - root.firstRow) * (root.cardHeight + root.rowGap) * card.u
                width: root.cardWidth * card.u
                height: root.cardHeight * card.u
                visible: root.visible && card.rowOf >= root.firstRow && card.rowOf < root.firstRow + root.visibleRows
                enabled: root.visible
                Binding {
                    target: card
                    property: "focus"
                    value: card.index === Settings.focused
                }
                onActiveFocusChanged: {
                    if (card.activeFocus)
                        Actions.settingsFocus(card.index)
                }
                onActivated: Actions.settingsChoose(card.index)
                Keys.onLeftPressed: (event) => event.accepted = card.colOf === 1 ? Actions.settingsFocus(card.index - 1) : false
                Keys.onRightPressed: (event) => event.accepted = card.colOf === 0 && card.index + 1 < Settings.count ? Actions.settingsFocus(card.index + 1) : false
                Keys.onUpPressed: (event) => event.accepted = card.index >= 2 ? Actions.settingsFocus(card.index - 2) : false
                Keys.onDownPressed: (event) => event.accepted = card.index + 2 < Settings.count ? Actions.settingsFocus(card.index + 2) : (card.colOf === 1 && card.index + 1 < Settings.count ? Actions.settingsFocus(card.index + 1) : false)
                Keys.onPressed: (event) => event.accepted = (event.key === Qt.Key_PageUp && Actions.settingsSection((Settings.section - 1 + Settings.sections.length) % Settings.sections.length)) || (event.key === Qt.Key_PageDown && Actions.settingsSection((Settings.section + 1) % Settings.sections.length))
                Rectangle {
                    objectName: "consoleFill." + card.index
                    anchors.fill: parent
                    radius: card.radius
                    color: card.lit ? Tokens.surfaceRaised : own.row
                    border.width: card.lit ? 0 : root.unit
                    border.color: own.rowLine
                }
                FocusRing {
                    objectName: "consoleRing"
                    anchors.fill: parent
                    visible: card.lit
                    radius: card.radius
                    gap: 3 * root.unit
                    thickness: 3 * root.unit
                    glowColor: Tokens.glowColor
                    glowBlur: 40 * root.unit
                    glowSpread: 8 * root.unit
                    bandRadius: card.radius + 6 * root.unit
                    pixelScale: root.pixelScale
                }
                PlanetSphere {
                    id: mark
                    objectName: "consoleMark." + card.index
                    unit: root.unit
                    planet: card.planet
                    side: 48
                    corner: 9
                    glowBlur: 18
                    glowSpread: 3
                    glowAlpha: 0x88 / 255
                    pixelScale: root.pixelScale
                    x: 24 * root.unit
                    y: (card.height - height) / 2
                }
                Chevron {
                    id: chevron
                    objectName: "consoleChevron." + card.index
                    unit: root.unit
                    ink: Tokens.muted
                    x: card.width - 24 * root.unit - width
                    y: (card.height - height) / 2
                }
                Rectangle {
                    id: badge
                    objectName: "consoleChip." + card.index
                    visible: card.chip.length > 0
                    x: chevron.x - 20 * root.unit - width
                    y: (card.height - height) / 2
                    width: badgeWords.width + 32 * root.unit
                    height: 36 * root.unit
                    radius: 18 * root.unit
                    color: card.chipAttention ? own.ringEnd : "transparent"
                    border.width: 2 * root.unit
                    border.color: card.chipAttention ? "transparent" : own.ringMuted
                    Line {
                        id: badgeWords
                        objectName: "consoleChipWords." + card.index
                        unit: root.unit
                        boardLine: true
                        x: (badge.width - width) / 2
                        y: (badge.height - height) / 2
                        text: card.chip
                        color: card.chipAttention ? Tokens.overAccent : Tokens.muted
                        size: 16
                        weight: Tokens.weightTitle
                    }
                }
                Column {
                    id: words
                    x: mark.x + mark.width + 20 * card.u
                    y: (card.height - height) / 2
                    width: Math.max(0, (badge.visible ? badge.x : chevron.x) - 20 * card.u - x)
                    spacing: 5 * card.u
                    Line {
                        objectName: "consoleName." + card.index
                        unit: root.unit
                        boardLine: true
                        width: parent.width
                        text: card.boardLabel
                        color: card.runs ? Tokens.text : own.unsupportedName
                        elide: Text.ElideRight
                        size: 25
                        weight: Tokens.weightTitle
                    }
                    Line {
                        objectName: "consoleSub." + card.index
                        unit: root.unit
                        boardLine: true
                        width: parent.width
                        text: card.subtitle
                        color: Tokens.muted
                        elide: Text.ElideRight
                        size: 18
                    }
                }
            }
        }
    }
}
