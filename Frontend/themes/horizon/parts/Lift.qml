import QtQuick
import EmuSen.Theme

// A strip item's look: lifted 12 and scaled 1.12 about its foot while focused, at rest while pressed, its content at 38% while the host would decline it, the ring 4 outside and 3 thick, breathing once still (EmuSen_Qt.md §3.4.3).
Item {
    id: root
    property bool lit: false
    property bool held: false
    property bool hovered: false
    property bool available: true
    // The box the ring goes round, in the item's units, and whether it is a circle.
    property real boxX: 0
    property real boxY: 0
    property real boxWidth: root.width
    property real boxHeight: root.height
    property bool round: false
    default property alias content: inside.data
    readonly property bool lifted: root.lit && !root.held
    readonly property bool resting: Tokens.breathes && !(Device.battery >= 0 && !Device.charging)
    Item {
        id: lifter
        anchors.fill: parent
        transformOrigin: Item.Bottom
        scale: root.lifted ? 1.12 : 1
        transform: Translate {
            y: root.lifted ? -12 : 0
            Behavior on y {
                NumberAnimation {
                    duration: Tokens.quick
                    easing.type: Easing.OutQuad
                }
            }
        }
        Behavior on scale {
            NumberAnimation {
                duration: Tokens.quick
                easing.type: Easing.OutQuad
            }
        }
        Item {
            id: inside
            anchors.fill: parent
            opacity: root.available ? 1 : 0.38
        }
        Rectangle {
            objectName: "hover"
            visible: root.hovered && !root.lit
            x: root.boxX
            y: root.boxY
            width: root.boxWidth
            height: root.boxHeight
            radius: root.round ? width / 2 : Tokens.tileRadius
            color: "transparent"
            border.width: 1
            border.color: Tokens.border
        }
        Rectangle {
            id: ring
            objectName: "ring"
            visible: root.lit
            x: root.boxX - Tokens.ringOffset - Tokens.ringThickness
            y: root.boxY - Tokens.ringOffset - Tokens.ringThickness
            width: root.boxWidth + 2 * (Tokens.ringOffset + Tokens.ringThickness)
            height: root.boxHeight + 2 * (Tokens.ringOffset + Tokens.ringThickness)
            radius: root.round ? width / 2 : Tokens.tileRadius + Tokens.ringOffset
            color: "transparent"
            border.width: Tokens.ringThickness
            border.color: Tokens.accent
            // The idle breath: after the delay, 100 to 70 to 100 % over the period, while focused and still.
            SequentialAnimation on opacity {
                running: root.lit && root.resting
                PauseAnimation {
                    duration: Tokens.breathDelay
                }
                SequentialAnimation {
                    loops: Animation.Infinite
                    NumberAnimation {
                        to: Tokens.breathLow
                        duration: Tokens.breathPeriod / 2
                        easing.type: Easing.InOutSine
                    }
                    NumberAnimation {
                        to: 1
                        duration: Tokens.breathPeriod / 2
                        easing.type: Easing.InOutSine
                    }
                }
            }
            onVisibleChanged: ring.opacity = 1
        }
    }
}
