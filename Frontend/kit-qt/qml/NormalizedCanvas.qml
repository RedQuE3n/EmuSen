import QtQuick

// The big screen's canvas: content laid out in 800 design units and scaled to the window's height, so a screen is 800 units high at every size (EmuSen_Interface.md §24.7, EmuSen_Qt.md §1.4).
Item {
    id: root
    property real designHeight: 800
    property bool active: true
    readonly property real unit: root.active && root.parent && root.parent.height > 0 ? root.parent.height / root.designHeight : 1
    width: root.parent ? root.parent.width / root.unit : 0
    height: root.active ? root.designHeight : (root.parent ? root.parent.height : 0)
    transform: Scale {
        xScale: root.unit
        yScale: root.unit
    }
}
