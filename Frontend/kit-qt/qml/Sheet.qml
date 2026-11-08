import QtQuick
import QtQuick.Controls
import EmuSen.Kit as Kit

// A sheet over the frame: a modal popup the controller offers presses to first; B closes it, the rest reach its focused control (EmuSen_Qt.md §1.7, EmuSen_Kit.md §7.8).
Popup {
    id: root
    required property Kit.FrameController controller
    signal pressed(int button)
    modal: true
    focus: true
    closePolicy: Popup.CloseOnEscape
    anchors.centerIn: Overlay.overlay
    width: Kit.Tokens.cardWidth
    onOpened: root.controller.sheetOpen = true
    onClosed: root.controller.sheetOpen = false
    Connections {
        target: root.controller
        function onSheetDismissed() { root.close() }
        function onSheetPressed(button) { root.pressed(button) }
    }
    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0; to: 1; duration: Kit.Tokens.enterFade; easing.type: Kit.Tokens.enterCurve }
    }
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1; to: 0; duration: Kit.Tokens.exitFade; easing.type: Kit.Tokens.exitCurve }
    }
}
