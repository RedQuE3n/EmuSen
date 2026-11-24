pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Theme

// A row of Eclipse's game page Details or Options tab as its boards draw it (EclipseGameDetails:64-77, EclipseGameOptions:64-77).
Walkable {
    id: root
    required property real unit
    required property int index
    required property string label
    required property string value
    required property bool steps
    required property bool opens
    required property bool isSwitch
    required property bool switchOn
    property real pixelScale: 1
    signal stepped(int by)
    readonly property bool lit: root.activeFocus
    readonly property real radius: 16 * root.unit
    readonly property real padX: 22 * root.unit
    objectName: "pageRow." + root.index
    Accessible.name: root.label
    Accessible.description: root.value
    height: 62 * root.unit
    Ink {
        id: ink
    }
    Rectangle {
        objectName: "pageRowFill." + root.index
        anchors.fill: parent
        radius: root.radius
        color: root.lit ? Tokens.surfaceRaised : Tokens.inputSurface
        border.width: root.lit ? 0 : root.unit
        border.color: ink.plateEdge
    }
    FocusRing {
        objectName: "pageRowRing." + root.index
        anchors.fill: parent
        visible: root.lit
        radius: root.radius
        gap: 4 * root.unit
        thickness: 3 * root.unit
        glowColor: ink.ringGlow
        glowBlur: 40 * root.unit
        glowSpread: 10 * root.unit
        bandRadius: (16 + 7) * root.unit
        pixelScale: root.pixelScale
    }
    Line {
        objectName: "pageRowName." + root.index
        unit: root.unit
        x: root.padX
        y: (root.height - height) / 2
        boardLine: true
        text: root.label
        color: Tokens.text
        size: 22
        weight: Font.DemiBold
    }
    Row {
        id: shown
        objectName: "pageRowValue." + root.index
        x: root.width - root.padX - width
        y: (root.height - height) / 2
        spacing: 12 * root.unit
        Chevron {
            objectName: "pageRowLess." + root.index
            unit: root.unit
            side: 18
            back: true
            visible: root.lit && root.steps
            ink: Tokens.accent
            y: (shown.height - height) / 2
            MouseArea {
                anchors.fill: parent
                onClicked: root.stepped(-1)
            }
        }
        Line {
            objectName: "pageRowWords." + root.index
            unit: root.unit
            visible: root.value.length > 0
            boardLine: true
            y: (shown.height - height) / 2
            text: root.value
            color: Tokens.text
            size: 21
            weight: Font.Normal
        }
        Chevron {
            objectName: "pageRowMore." + root.index
            unit: root.unit
            side: 18
            back: false
            visible: root.lit && root.steps
            ink: Tokens.accent
            y: (shown.height - height) / 2
            MouseArea {
                anchors.fill: parent
                onClicked: root.stepped(1)
            }
        }
        Chevron {
            objectName: "pageRowDoor." + root.index
            unit: root.unit
            side: 18
            back: false
            visible: root.opens
            ink: Tokens.muted
            y: (shown.height - height) / 2
        }
        Rectangle {
            objectName: "pageRowSwitch." + root.index
            visible: root.isSwitch
            y: (shown.height - height) / 2
            width: 68 * root.unit
            height: 38 * root.unit
            radius: 19 * root.unit
            color: root.switchOn ? Tokens.accent : ink.plateEdge
            Rectangle {
                objectName: "pageRowKnob." + root.index
                x: (root.switchOn ? 34 : 4) * root.unit
                y: 4 * root.unit
                width: 30 * root.unit
                height: 30 * root.unit
                radius: 15 * root.unit
                color: root.switchOn ? Tokens.overAccent : ink.emptyLine
            }
        }
    }
}
