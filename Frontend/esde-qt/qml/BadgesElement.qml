pragma ComponentBehavior: Bound
import QtQuick
import EmuSen.Esde

// The game's badges in the cells the scene laid out, each with its controller and folder-link overlays.
ElementBox {
    id: root
    required property bool nearest
    PartsModel {
        id: parts
        scene: root.esde
        revision: root.esde ? root.esde.revision : 0
        element: root.elementId
    }
    Repeater {
        model: parts
        Item {
            id: badge
            required property url iconSource
            required property string iconMark
            required property real iconX
            required property real iconY
            required property real iconW
            required property real iconH
            required property color iconTint
            required property color iconTintEnd
            required property bool iconVertical
            required property url controllerSource
            required property string controllerMark
            required property real controllerX
            required property real controllerY
            required property real controllerW
            required property real controllerH
            required property color controllerTint
            required property color controllerTintEnd
            required property bool controllerVertical
            required property url linkSource
            required property string linkMark
            required property real linkX
            required property real linkY
            required property real linkW
            required property real linkH
            required property color linkTint
            required property color linkTintEnd
            required property bool linkVertical
            anchors.fill: parent
            Icon {
                x: badge.iconX
                y: badge.iconY
                width: badge.iconW
                height: badge.iconH
                source: badge.iconSource
                mark: badge.iconMark
                tint: badge.iconTint
                tintEnd: badge.iconTintEnd
                gradientVertical: badge.iconVertical
                nearest: root.nearest
            }
            Icon {
                x: badge.controllerX
                y: badge.controllerY
                width: badge.controllerW
                height: badge.controllerH
                source: badge.controllerSource
                mark: badge.controllerMark
                tint: badge.controllerTint
                tintEnd: badge.controllerTintEnd
                gradientVertical: badge.controllerVertical
                nearest: root.nearest
            }
            Icon {
                x: badge.linkX
                y: badge.linkY
                width: badge.linkW
                height: badge.linkH
                source: badge.linkSource
                mark: badge.linkMark
                tint: badge.linkTint
                tintEnd: badge.linkTintEnd
                gradientVertical: badge.linkVertical
                nearest: root.nearest
            }
        }
    }
}
