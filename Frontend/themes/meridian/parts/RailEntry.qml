import QtQuick
import EmuSen.Theme

// One entry of the rail, 264 x 54: the raised fill and the Accent bar while focused (dimmed while the band holds the focus), a mark or a world at 28, the name and its line (meridian.md §1.2, §1.7, §2).
Walkable {
    id: entry
    property string name
    property string line
    property bool attention
    // A drawn mark of Marks, or a console's world drawn by Body when `world` is set.
    property string mark
    property string world
    property real hue: -1
    // The band holds the focus, so this entry keeps a dimmed mark of where it was.
    property bool held
    // Disabled: content at 38%, the bar and the name kept.
    property bool available: true
    property bool pressed
    property bool compact
    required property Faces faces
    readonly property bool marked: entry.activeFocus || entry.held
    readonly property bool light: Tokens.variant === Tokens.Light
    width: entry.compact ? 64 : 264
    height: 54
    Accessible.name: entry.name
    Accessible.description: entry.line
    // A held press widens the bar to 6; the press still reaches Walkable's own handler.
    Keys.onPressed: (event) => {
        entry.pressed = event.key === Qt.Key_Return || event.key === Qt.Key_Enter
        event.accepted = false
    }
    Keys.onReleased: entry.pressed = false
    onActiveFocusChanged: entry.pressed = false
    Scene {
        anchors.fill: parent
        Rectangle {
            objectName: "entryFill"
            anchors.fill: parent
            radius: Tokens.rowRadius
            visible: entry.marked
            color: entry.light ? "#FFFFFF" : Tokens.surfaceRaised
            opacity: (entry.light ? 1 : 0.92) * (entry.held && !entry.activeFocus ? 0.5 : 1)
            Behavior on opacity {
                NumberAnimation {
                    duration: Tokens.quick
                    easing.type: Tokens.quickCurve
                }
            }
        }
    }
    Rectangle {
        objectName: "entryBar"
        visible: entry.marked
        width: entry.pressed ? 6 : Tokens.barWidth
        height: parent.height
        radius: Tokens.rowRadius
        color: Tokens.accent
        opacity: (entry.held && !entry.activeFocus ? 0.5 : 1) * (entry.available ? 1 : 0.38)
    }
    Item {
        anchors.fill: parent
        opacity: entry.available ? 1 : 0.38
        Glyph {
            objectName: "entryMark"
            visible: entry.world.length === 0
            x: entry.compact ? 20 : 18
            y: 15
            width: 24
            height: 24
            path: entry.mark
            ink: entry.activeFocus ? Tokens.textStrong : Tokens.text
        }
        Body {
            objectName: "entryWorld"
            visible: entry.world.length > 0
            x: entry.compact ? 18 : 16
            y: 13
            width: 28
            height: 28
            world: entry.world
            hue: entry.hue
        }
        Text {
            objectName: "entryName"
            visible: !entry.compact
            x: 56
            y: 4
            width: 196
            text: entry.name
            elide: Text.ElideRight
            color: entry.activeFocus ? Tokens.textStrong : Tokens.text
            font.family: entry.activeFocus ? (entry.faces?.medium ?? "") : (entry.faces?.regular ?? "")
            font.pixelSize: 22
            lineHeight: 26.4
            lineHeightMode: Text.FixedHeight
        }
        Row {
            visible: !entry.compact
            x: 56
            y: 30
            spacing: 4
            Glyph {
                objectName: "entryAttention"
                visible: entry.attention
                anchors.verticalCenter: parent.verticalCenter
                width: 14
                height: 14
                path: Marks.attention
                filled: true
                ink: Tokens.attention
            }
            Text {
                objectName: "entryLine"
                // The line takes the entry's whole width past the name's inset.
                width: entry.width - 64 - (entry.attention ? 18 : 0)
                text: entry.line
                elide: Text.ElideRight
                color: entry.attention ? Tokens.attention : Tokens.muted
                font.family: (entry.faces?.regular ?? "")
                font.pixelSize: 19
                lineHeight: 22.8
                lineHeightMode: Text.FixedHeight
            }
        }
    }
}
