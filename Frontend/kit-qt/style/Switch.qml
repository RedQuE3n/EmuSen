import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A switch in the tokens: a track that fills with the accent, a knob in overAccent, the ring on focus.
T.Switch {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding)
    padding: Tokens.base * 2
    spacing: Tokens.base * 2
    font.pixelSize: Tokens.fontBody
    indicator: Rectangle {
        implicitWidth: Tokens.controlHeight * 1.3
        implicitHeight: Tokens.controlHeight * 0.65
        x: control.text ? (control.mirrored ? control.width - width - control.rightPadding : control.leftPadding) : control.leftPadding + (control.availableWidth - width) / 2
        y: control.topPadding + (control.availableHeight - height) / 2
        radius: height / 2
        color: control.checked ? Tokens.accent : Tokens.inputSurface
        border.width: control.activeFocus ? Tokens.ringThickness : Tokens.hoverBorder
        border.color: control.activeFocus ? Tokens.accent : Tokens.border
        Rectangle {
            x: Math.max(0, Math.min(parent.width - width, control.visualPosition * parent.width - (width / 2)))
            y: (parent.height - height) / 2
            width: parent.height - Tokens.base
            height: width
            radius: width / 2
            color: control.checked ? Tokens.overAccent : Tokens.textStrong
            Behavior on x {
                enabled: !control.down
                NumberAnimation { duration: Tokens.quick; easing.type: Tokens.quickCurve }
            }
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
