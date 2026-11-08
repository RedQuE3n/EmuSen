import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A check box in the tokens: an input-surface box, filled with the accent and a mark in overAccent when checked.
T.CheckBox {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding)
    padding: Tokens.base * 2
    spacing: Tokens.base * 2
    font.pixelSize: Tokens.fontBody
    indicator: Rectangle {
        implicitWidth: Tokens.fontBody * 1.4
        implicitHeight: Tokens.fontBody * 1.4
        x: control.text ? (control.mirrored ? control.width - width - control.rightPadding : control.leftPadding) : control.leftPadding + (control.availableWidth - width) / 2
        y: control.topPadding + (control.availableHeight - height) / 2
        radius: Tokens.rowRadius / 2
        color: control.checkState !== Qt.Unchecked ? Tokens.accent : Tokens.inputSurface
        border.width: control.activeFocus ? Tokens.ringThickness : Tokens.hoverBorder
        border.color: control.activeFocus ? Tokens.accent : Tokens.border
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 0.5
            height: control.checkState === Qt.PartiallyChecked ? parent.height * 0.2 : width
            radius: Tokens.rowRadius / 2
            color: Tokens.overAccent
            visible: control.checkState !== Qt.Unchecked
        }
    }
    contentItem: CheckLabel {
        leftPadding: control.indicator && !control.mirrored ? control.indicator.width + control.spacing : 0
        rightPadding: control.indicator && control.mirrored ? control.indicator.width + control.spacing : 0
        text: control.text
        font: control.font
        color: control.enabled ? Tokens.text : Tokens.muted
    }
}
