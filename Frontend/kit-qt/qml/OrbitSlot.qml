import QtQuick
import EmuSen.Kit as Kit

// A cover of a list laid on an orbit about (the view's centre, cy): moved to its point on the ellipse by its distance from `turning`, tilted by the ellipse's normal, scaled down by distance and lifted 12 at the apex when lit; the covers of one list are a stack, each over the next by its nearness to the apex (EmuSen_Kit.md §7.33).
Kit.Walkable {
    id: slot
    required property ListView view
    // The list's current index followed over the theme's motion, so a turn carries every cover along the arc together.
    required property real turning
    required property real rx
    required property real ry
    required property real cy
    required property real step
    required property real coverHeight
    required property real scaleStep
    required property real minScale
    required property real coverShape
    required property int index
    property bool lit
    readonly property real d: slot.index - slot.turning
    readonly property real far: Math.abs(slot.d)
    readonly property real theta: (90 - slot.step * slot.d) * Math.PI / 180
    readonly property real size: Math.max(slot.minScale, 1 - slot.scaleStep * slot.far)
    readonly property real tilt: -Math.atan2(Math.cos(slot.theta) / slot.rx, Math.sin(slot.theta) / slot.ry) * 180 / Math.PI
    width: Math.round(slot.coverHeight * Math.min(1.6, Math.max(0.5, slot.coverShape > 0 ? slot.coverShape : 0.75)))
    height: slot.coverHeight
    z: 10 - slot.far
    // The fourth from the apex exists unseen, so it fades in as the orbit turns.
    opacity: Math.max(0, Math.min(1, 3 - slot.far))
    visible: slot.far < 3.5
    transform: [
        Scale {
            origin.x: slot.width / 2
            origin.y: slot.height / 2
            xScale: slot.size
            yScale: slot.size
        },
        Rotation {
            origin.x: slot.width / 2
            origin.y: slot.height / 2
            angle: slot.tilt
        },
        // A deleted screen's view is gone before its covers, which Qt deletes later (EmuSen_Kit.md §7.40).
        Translate {
            x: (slot.view?.width ?? 0) / 2 + slot.rx * Math.cos(slot.theta) - (slot.x - (slot.view?.contentX ?? 0) + slot.width / 2)
            y: slot.cy + slot.ry * Math.sin(slot.theta) - (slot.view?.y ?? 0) - slot.height / 2 - (slot.lit ? 12 : 0)
        }
    ]
    Kit.StackMember {}
}
