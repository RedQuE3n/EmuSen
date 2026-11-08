import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A tab in the tokens: strong text when current, muted otherwise, the accent bar under the current tab, the ring on focus.
T.TabButton {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    padding: Tokens.base * 2
    horizontalPadding: Tokens.base * 4
    spacing: Tokens.base * 2
    font.pixelSize: Tokens.fontBody
    icon.width: Tokens.fontBody * 1.5
    icon.height: Tokens.fontBody * 1.5
    contentItem: IconLabel {
        spacing: control.spacing
        mirrored: control.mirrored
        display: control.display
        icon: control.icon
        text: control.text
        font: control.font
        color: control.checked ? Tokens.textStrong : (control.enabled ? Tokens.muted : Tokens.border)
    }
    background: Rectangle {
        implicitHeight: Tokens.controlHeight
        radius: Tokens.rowRadius
        color: control.down || control.hovered ? Tokens.surfaceRaised : Tokens.surface
        border.width: control.activeFocus ? Tokens.ringThickness : 0
        border.color: Tokens.accent
        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: Tokens.barWidth
            color: Tokens.accent
            visible: control.checked
        }
    }
}
