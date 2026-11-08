import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A popup in the tokens: the raised surface with a border and the tile radius, the scrim behind a modal one.
T.Popup {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding)
    padding: Tokens.paneGutter
    background: Rectangle {
        radius: Tokens.tileRadius
        color: Tokens.surfaceRaised
        border.width: Tokens.hoverBorder
        border.color: Tokens.border
    }
    T.Overlay.modal: Rectangle {
        color: Tokens.scrim
    }
    T.Overlay.modeless: Rectangle {
        color: Tokens.scrim
        opacity: 0.25
    }
}
