import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A popup in the tokens: the raised surface with a border and the tile radius, the scrim behind a modal one.
T.Popup {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding)
    padding: Kit.Tokens.paneGutter
    background: Rectangle {
        radius: Kit.Tokens.tileRadius
        color: Kit.Tokens.surfaceRaised
        border.width: Kit.Tokens.hoverBorder
        border.color: Kit.Tokens.border
    }
    T.Overlay.modal: Rectangle {
        color: Kit.Tokens.scrim
    }
    T.Overlay.modeless: Rectangle {
        color: Kit.Tokens.scrim
        opacity: 0.25
    }
}
