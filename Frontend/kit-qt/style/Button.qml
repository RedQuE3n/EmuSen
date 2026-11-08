import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A button in the tokens: raised surface, the accent when highlighted or checked, the ring on focus (EmuSen_Qt.md §1.5).
T.Button {
    id: control
    Keys.onReturnPressed: control.clicked()
    Keys.onEnterPressed: control.clicked()
    implicitWidth: Math.ceil(Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding))
    implicitHeight: Math.ceil(Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding))
    padding: Kit.Tokens.base * 2
    horizontalPadding: Kit.Tokens.base * 4
    spacing: Kit.Tokens.base * 2
    font.pixelSize: Kit.Tokens.fontBody
    icon.width: Kit.Tokens.fontBody * 1.5
    icon.height: Kit.Tokens.fontBody * 1.5
    contentItem: IconLabel {
        spacing: control.spacing
        mirrored: control.mirrored
        display: control.display
        icon: control.icon
        text: control.text
        font: control.font
        color: control.highlighted || control.checked ? Kit.Tokens.overAccent : (control.enabled ? Kit.Tokens.textStrong : Kit.Tokens.muted)
    }
    background: Rectangle {
        implicitWidth: Kit.Tokens.controlHeight * 2.5
        implicitHeight: Kit.Tokens.controlHeight
        radius: Kit.Tokens.rowRadius
        visible: !control.flat || control.down || control.checked || control.highlighted || control.hovered
        color: control.highlighted || control.checked ? Kit.Tokens.accent : (control.down ? Kit.Tokens.inputSurface : Kit.Tokens.surfaceRaised)
        border.width: control.activeFocus ? Kit.Tokens.ringThickness : (control.hovered ? Kit.Tokens.hoverBorder : 0)
        border.color: control.activeFocus ? Kit.Tokens.accent : Kit.Tokens.border
    }
}
