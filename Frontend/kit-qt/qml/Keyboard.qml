pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Kit as Kit

// The docked keyboard: a band of keys on Surface under a rule, the current key ringed while the band holds the focus; the arrows walk the keys, A types, B erases then closes, a pointer rests on a key and clicks it (EmuSen_Interface.md §18.1, EmuSen_Kit.md §7.22).
FocusScope {
    id: root
    required property Kit.KeyboardModel model
    // The keys' words' face; the reference theme sets Barlow.
    property font labelFont: Qt.font({ pixelSize: Kit.Tokens.fontBody })
    readonly property bool showsCurrent: root.activeFocus
    implicitHeight: root.model.bandHeight
    activeFocusOnTab: true
    Accessible.role: Accessible.Grouping
    Accessible.name: root.model.text
    Keys.onPressed: (event) => event.accepted = root.model.key(event.key, event.modifiers, event.text)
    // A click anywhere on the band gives it the focus; a click on a key also types it.
    TapHandler {
        onTapped: root.forceActiveFocus()
    }
    Rectangle {
        objectName: "keyboardBand"
        anchors.fill: parent
        color: Kit.Tokens.surface
    }
    Rectangle {
        objectName: "keyboardRule"
        width: parent.width
        height: Kit.Tokens.hoverBorder
        color: Kit.Tokens.border
    }
    Repeater {
        model: root.model
        Rectangle {
            id: key
            required property int index
            required property string label
            required property real keyX
            required property real keyY
            required property real keyWidth
            required property real keyHeight
            readonly property bool ringed: root.showsCurrent && key.index === root.model.current
            objectName: "key." + key.index
            x: root.width / 2 + key.keyX
            y: key.keyY
            width: key.keyWidth
            height: key.keyHeight
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.surfaceRaised
            border.width: key.ringed ? Kit.Tokens.hoverBorder * 2 : 0
            border.color: Kit.Tokens.accent
            Accessible.role: Accessible.Button
            Accessible.name: key.label
            Text {
                anchors.centerIn: parent
                text: key.label
                color: key.ringed ? Kit.Tokens.textStrong : Kit.Tokens.text
                font: root.labelFont
            }
            HoverHandler {
                onHoveredChanged: root.model.point(hovered ? key.index : -1)
            }
            TapHandler {
                onTapped: root.model.click(key.index)
            }
        }
    }
}
