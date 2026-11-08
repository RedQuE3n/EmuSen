import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A pane in the tokens: the surface with the pane gutter.
T.Pane {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding)
    padding: Kit.Tokens.paneGutter
    background: Rectangle {
        color: Kit.Tokens.surface
    }
}
