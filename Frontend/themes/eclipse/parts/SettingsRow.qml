pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// One of Settings' rows as the boards draw it: its name over its short line, and its value in one of four forms, a choice, a switch, a door or a fact (G:26-48, 52-63; EmuSen_Qt.md §3.4.11).
Walkable {
    id: root
    // A board pixel in canvas units, Board.unit.
    required property real unit
    required property int index
    required property string label
    required property string about
    required property string value
    required property int kind
    required property bool isSwitch
    required property bool switchOn
    required property bool code
    // The value's form, the model's (Settings.rows' form, 1.11): Settings.Choice, Switch, Door or Fact.
    required property int form
    // Scene pixels to a canvas unit, so the ring's glow blurs at the pixels drawn.
    property real pixelScale: 1
    // Left or Right on a chevron, which steps the value.
    signal stepped(int by)
    readonly property bool choice: root.form === Settings.Choice
    readonly property bool door: root.form === Settings.Door
    readonly property bool fact: root.form === Settings.Fact
    readonly property bool lit: root.activeFocus
    readonly property real radius: 20 * root.unit
    // The boards' row: 84 high at least, 12 by 24 inside, its two lines 4 apart (G:27-28).
    readonly property real padX: 24
    readonly property real padY: 12
    readonly property real least: 84
    objectName: "row." + root.index
    Accessible.name: root.label
    height: Math.max(root.least * root.unit, words.height + 2 * root.padY * root.unit)
    SettingsInk {
        id: ink
    }
    Ink {
        id: shared
    }
    LineBox {
        id: nameBox
        size: 24
        weight: Tokens.weightTitle
    }
    LineBox {
        id: aboutBox
        size: 17
    }
    LineBox {
        id: valueBox
        size: 22
    }
    Rectangle {
        objectName: "rowFill." + root.index
        anchors.fill: parent
        radius: root.radius
        color: root.lit ? Tokens.surfaceRaised : ink.row
        // A row at rest has its 1 px line inside its edge (G:63).
        border.width: root.lit ? 0 : root.unit
        border.color: ink.rowLine
    }
    // The focused row's ring: 3 px of the ground's dark, 3 px of the accent, and 40 px of glow 8 px out (G:63).
    FocusRing {
        objectName: "rowRing"
        anchors.fill: parent
        visible: root.lit
        radius: root.radius
        gap: 3 * root.unit
        thickness: 3 * root.unit
        glowColor: Tokens.glowColor
        glowBlur: 40 * root.unit
        glowSpread: 8 * root.unit
        bandRadius: root.radius + 6 * root.unit
        pixelScale: root.pixelScale
    }
    Column {
        id: words
        x: root.padX * root.unit
        y: (root.height - height) / 2
        spacing: 4 * root.unit
        Line {
            objectName: "rowName." + root.index
            unit: root.unit
            height: nameBox.box * root.unit
            text: root.label
            color: Tokens.text
            size: 24
            weight: Tokens.weightTitle
        }
        // The short line, cut where the value needs the room, as the board's ellipsis does (G:30).
        Line {
            objectName: "rowAbout." + root.index
            unit: root.unit
            visible: root.about.length > 0
            width: Math.min(implicitWidth, root.width - 2 * root.padX * root.unit - 24 * root.unit - shown.width)
            height: aboutBox.box * root.unit
            text: root.about
            color: Tokens.muted
            elide: Text.ElideRight
            size: 17
        }
    }
    Row {
        id: shown
        objectName: "rowValue." + root.index
        x: root.width - root.padX * root.unit - width
        y: (root.height - height) / 2
        spacing: 14 * root.unit
        Chevron {
            objectName: "rowLess." + root.index
            unit: root.unit
            back: true
            visible: root.choice
            ink: Tokens.accent
            y: (shown.height - height) / 2
            MouseArea {
                anchors.fill: parent
                onClicked: root.stepped(-1)
            }
        }
        Line {
            objectName: "rowWords." + root.index
            unit: root.unit
            visible: !root.fact || root.value.length > 0
            width: Math.min(implicitWidth, 420 * root.unit)
            height: valueBox.box * root.unit
            y: (shown.height - height) / 2
            text: root.value
            color: root.choice || root.isSwitch ? Tokens.text : Tokens.muted
            family: root.code ? Tokens.faceCode : Tokens.faceText
            elide: Text.ElideRight
            size: 22
        }
        Chevron {
            objectName: "rowMore." + root.index
            unit: root.unit
            visible: root.choice
            ink: Tokens.accent
            y: (shown.height - height) / 2
            MouseArea {
                anchors.fill: parent
                onClicked: root.stepped(1)
            }
        }
        // The switch: a 68 by 38 track inside its 2 px line, its 26 px knob 4 px in at the side it is turned to (G:41).
        Rectangle {
            objectName: "rowSwitch." + root.index
            visible: root.isSwitch
            y: (shown.height - height) / 2
            width: 68 * root.unit
            height: 38 * root.unit
            radius: height / 2
            color: root.switchOn ? Tokens.accent : "transparent"
            border.width: 2 * root.unit
            border.color: root.switchOn ? Tokens.accent : shared.pillLine
            Rectangle {
                objectName: "rowKnob." + root.index
                x: root.switchOn ? parent.width - (2 + 4 + 26) * root.unit : (2 + 4) * root.unit
                y: (parent.height - height) / 2
                width: 26 * root.unit
                height: width
                radius: width / 2
                color: root.switchOn ? Tokens.overAccent : Tokens.muted
            }
        }
        Chevron {
            objectName: "rowDoor." + root.index
            unit: root.unit
            visible: root.door
            ink: Tokens.muted
            y: (shown.height - height) / 2
        }
    }
}
