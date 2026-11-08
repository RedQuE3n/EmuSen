import QtQuick
import QtQuick.Controls
import EmuSen.Kit

// A sheet over the frame: a modal popup the controller offers presses to first; B closes it, the rest reach its focused control (EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.8).
Popup {
    id: root
    required property FrameController controller
    signal pressed(int button)
    modal: true
    focus: true
    closePolicy: Popup.CloseOnEscape
    anchors.centerIn: Overlay.overlay
    width: Tokens.cardWidth
    onOpened: root.controller.sheetOpen = true
    onClosed: root.controller.sheetOpen = false
    Connections {
        target: root.controller
        function onSheetDismissed() { root.close() }
        function onSheetPressed(button) { root.pressed(button) }
    }
    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Tokens.enterFade; easing.type: Tokens.enterCurve }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1; to: 0; duration: Tokens.exitFade; easing.type: Tokens.exitCurve }
    }
}
