import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A button in the tokens: raised surface, the accent when highlighted or checked, the ring on focus (EmuSen_Qt.md §1.5).
T.Button {
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
        color: control.highlighted || control.checked ? Tokens.overAccent : (control.enabled ? Tokens.textStrong : Tokens.muted)
    }
    background: Rectangle {
        implicitWidth: Tokens.controlHeight * 2.5
        implicitHeight: Tokens.controlHeight
        radius: Tokens.rowRadius
        visible: !control.flat || control.down || control.checked || control.highlighted || control.hovered
        color: control.highlighted || control.checked ? Tokens.accent : (control.down ? Tokens.inputSurface : Tokens.surfaceRaised)
        border.width: control.activeFocus ? Tokens.ringThickness : (control.hovered ? Tokens.hoverBorder : 0)
        border.color: control.activeFocus ? Tokens.accent : Tokens.border
    }
}
