import QtQuick
import EmuSen.Kit as Kit

// An element the frame walks to: hovering focuses it, a click is A, a right click or the back button is B (EmuSen_Qt.md §1.7). Declared once here, never per screen; a scope, so content asking for the focus gets it.
FocusScope {
    id: root
    required property Kit.FrameController controller
    signal activated()
    readonly property alias hovered: hover.hovered
    activeFocusOnTab: true
    Accessible.focusable: true
    // A is heard first by the element's own Keys.onPressed, a theme's, which may take it; else it activates (EmuSen_Kit.md §7.37).
    Keys.onReturnPressed: (event) => {
        event.accepted = false
        root.Keys.pressed(event)
        if (!event.accepted)
            root.activated()
        event.accepted = true
    }
    Keys.onEnterPressed: (event) => root.Keys.returnPressed(event)
    HoverHandler {
        id: hover
        onHoveredChanged: root.controller.hover(root, hovered)
    }
    TapHandler {
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.BackButton
        onTapped: (point, button) => root.controller.click(root, button)
    }
}
