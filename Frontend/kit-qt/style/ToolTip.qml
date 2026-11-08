import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A tool tip in the tokens: strong text on the raised surface with a border.
T.ToolTip {
    id: control
    x: parent ? (parent.width - implicitWidth) / 2 : 0
    y: -implicitHeight - Tokens.base
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    margins: Tokens.base
    padding: Tokens.base * 2
    font.pixelSize: Tokens.fontCaption
    closePolicy: T.Popup.CloseOnEscape | T.Popup.CloseOnPressOutsideParent | T.Popup.CloseOnReleaseOutsideParent
    contentItem: Text {
        text: control.text
        font: control.font
        wrapMode: Text.Wrap
        color: Tokens.textStrong
    }
    background: Rectangle {
        radius: Tokens.rowRadius
        color: Tokens.surfaceRaised
        border.width: Tokens.hoverBorder
        border.color: Tokens.border
    }
}
