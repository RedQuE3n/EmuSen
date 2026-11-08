import QtQuick
import EmuSen.Kit

// An element the frame walks to: hovering focuses it, a click is A, a right click or the back button is B (EmuSen_Qt.md §1.7). Declared once here, never per screen.
Item {
    id: root
    required property FrameController controller
    signal activated()
    readonly property alias hovered: hover.hovered
    activeFocusOnTab: true
    Accessible.focusable: true
    Keys.onReturnPressed: root.activated()
    Keys.onEnterPressed: root.activated()
    HoverHandler {
        id: hover
        onHoveredChanged: root.controller.hover(root, hovered)
    }
    TapHandler {
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.BackButton
        onTapped: (point, button) => root.controller.click(root, button)
    }
}
