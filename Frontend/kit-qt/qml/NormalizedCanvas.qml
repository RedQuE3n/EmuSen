import QtQuick
import EmuSen.Kit as Kit

// The big screen's canvas: content laid out in the design height's units and scaled to the window's height, the height kit-core owns (EmuSen_Kit.md §7.112, EmuSen_Qt.md §1.4).
Item {
    id: root
    property real designHeight: Kit.CanvasUnits.designHeight
    property bool active: true
    readonly property real unit: root.active && root.parent && root.parent.height > 0 ? root.parent.height / root.designHeight : 1
    width: root.parent ? root.parent.width / root.unit : 0
    height: root.active ? root.designHeight : (root.parent ? root.parent.height : 0)
    transform: Scale {
        xScale: root.unit
        yScale: root.unit
    }
}
