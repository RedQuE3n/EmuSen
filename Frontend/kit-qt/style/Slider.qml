import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A slider in the tokens: an input-surface track filled with the accent to the value, a round handle, the ring on focus.
T.Slider {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitHandleWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitHandleHeight + topPadding + bottomPadding)
    padding: Kit.Tokens.base * 2
    handle: Rectangle {
        x: control.leftPadding + (control.horizontal ? control.visualPosition * (control.availableWidth - width) : (control.availableWidth - width) / 2)
        y: control.topPadding + (control.horizontal ? (control.availableHeight - height) / 2 : control.visualPosition * (control.availableHeight - height))
        implicitWidth: Kit.Tokens.controlHeight * 0.6
        implicitHeight: Kit.Tokens.controlHeight * 0.6
        radius: width / 2
        color: control.pressed ? Kit.Tokens.overAccent : Kit.Tokens.textStrong
        border.width: control.activeFocus ? Kit.Tokens.ringThickness : Kit.Tokens.hoverBorder
        border.color: control.activeFocus ? Kit.Tokens.accent : Kit.Tokens.border
    }
    background: Rectangle {
        x: control.leftPadding + (control.horizontal ? 0 : (control.availableWidth - width) / 2)
        y: control.topPadding + (control.horizontal ? (control.availableHeight - height) / 2 : 0)
        implicitWidth: control.horizontal ? Kit.Tokens.controlHeight * 5 : Kit.Tokens.base * 1.5
        implicitHeight: control.horizontal ? Kit.Tokens.base * 1.5 : Kit.Tokens.controlHeight * 5
        width: control.horizontal ? control.availableWidth : implicitWidth
        height: control.horizontal ? implicitHeight : control.availableHeight
        radius: height / 2
        color: Kit.Tokens.inputSurface
        scale: control.horizontal && control.mirrored ? -1 : 1
        Rectangle {
            y: control.horizontal ? 0 : control.visualPosition * parent.height
            width: control.horizontal ? control.position * parent.width : parent.width
            height: control.horizontal ? parent.height : control.position * parent.height
            radius: parent.radius
            color: Kit.Tokens.accent
        }
    }
}
