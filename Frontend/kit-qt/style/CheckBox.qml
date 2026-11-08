import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A check box in the tokens: an input-surface box, filled with the accent and a mark in overAccent when checked.
T.CheckBox {
    id: control
    Keys.onReturnPressed: control.toggle()
    Keys.onEnterPressed: control.toggle()
    implicitWidth: Math.ceil(Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding))
    implicitHeight: Math.ceil(Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding))
    padding: Kit.Tokens.base * 2
    spacing: Kit.Tokens.base * 2
    font.pixelSize: Kit.Tokens.fontBody
    indicator: Rectangle {
        implicitWidth: Kit.Tokens.fontBody * 1.4
        implicitHeight: Kit.Tokens.fontBody * 1.4
        x: control.text ? (control.mirrored ? control.width - width - control.rightPadding : control.leftPadding) : control.leftPadding + (control.availableWidth - width) / 2
        y: control.topPadding + (control.availableHeight - height) / 2
        radius: Kit.Tokens.rowRadius / 2
        color: control.checkState !== Qt.Unchecked ? Kit.Tokens.accent : Kit.Tokens.inputSurface
        border.width: control.activeFocus ? Kit.Tokens.ringThickness : Kit.Tokens.hoverBorder
        border.color: control.activeFocus ? Kit.Tokens.accent : Kit.Tokens.border
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 0.5
            height: control.checkState === Qt.PartiallyChecked ? parent.height * 0.2 : width
            radius: Kit.Tokens.rowRadius / 2
            color: Kit.Tokens.overAccent
            visible: control.checkState !== Qt.Unchecked
        }
    }
    contentItem: CheckLabel {
        leftPadding: control.indicator && !control.mirrored ? control.indicator.width + control.spacing : 0
        rightPadding: control.indicator && control.mirrored ? control.indicator.width + control.spacing : 0
        text: control.text
        font: control.font
        color: control.enabled ? Kit.Tokens.text : Kit.Tokens.muted
    }
}
