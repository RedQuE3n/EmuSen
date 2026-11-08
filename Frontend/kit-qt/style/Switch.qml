import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A switch in the tokens: a track that fills with the accent, a knob in overAccent, the ring on focus.
T.Switch {
    id: control
    Keys.onReturnPressed: control.toggle()
    Keys.onEnterPressed: control.toggle()
    implicitWidth: Math.ceil(Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding))
    implicitHeight: Math.ceil(Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding))
    padding: Kit.Tokens.base * 2
    spacing: Kit.Tokens.base * 2
    font.pixelSize: Kit.Tokens.fontBody
    indicator: Rectangle {
        implicitWidth: Kit.Tokens.controlHeight * 1.3
        implicitHeight: Kit.Tokens.controlHeight * 0.65
        x: control.text ? (control.mirrored ? control.width - width - control.rightPadding : control.leftPadding) : control.leftPadding + (control.availableWidth - width) / 2
        y: control.topPadding + (control.availableHeight - height) / 2
        radius: height / 2
        color: control.checked ? Kit.Tokens.accent : Kit.Tokens.inputSurface
        border.width: control.activeFocus ? Kit.Tokens.ringThickness : Kit.Tokens.hoverBorder
        border.color: control.activeFocus ? Kit.Tokens.accent : Kit.Tokens.border
        Rectangle {
            x: Math.max(0, Math.min(parent.width - width, control.visualPosition * parent.width - (width / 2)))
            y: (parent.height - height) / 2
            width: parent.height - Kit.Tokens.base
            height: width
            radius: width / 2
            color: control.checked ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
            Behavior on x {
                enabled: !control.down
                NumberAnimation { duration: Kit.Tokens.quick; easing.type: Kit.Tokens.quickCurve }
            }
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
