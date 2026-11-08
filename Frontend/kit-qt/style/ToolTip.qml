import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A tool tip in the tokens: strong text on the raised surface with a border.
T.ToolTip {
    id: control
    x: parent ? (parent.width - implicitWidth) / 2 : 0
    y: -implicitHeight - Kit.Tokens.base
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    margins: Kit.Tokens.base
    padding: Kit.Tokens.base * 2
    font.pixelSize: Kit.Tokens.fontCaption
    closePolicy: T.Popup.CloseOnEscape | T.Popup.CloseOnPressOutsideParent | T.Popup.CloseOnReleaseOutsideParent
    contentItem: Text {
        text: control.text
        font: control.font
        wrapMode: Text.Wrap
        color: Kit.Tokens.textStrong
    }
    background: Rectangle {
        radius: Kit.Tokens.rowRadius
        color: Kit.Tokens.surfaceRaised
        border.width: Kit.Tokens.hoverBorder
        border.color: Kit.Tokens.border
    }
}
