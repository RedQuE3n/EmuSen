import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A slider in the tokens: an input-surface track filled with the accent to the value, a round handle, the ring on focus.
T.Slider {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitHandleWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitHandleHeight + topPadding + bottomPadding)
    padding: Tokens.base * 2
    handle: Rectangle {
        x: control.leftPadding + (control.horizontal ? control.visualPosition * (control.availableWidth - width) : (control.availableWidth - width) / 2)
        y: control.topPadding + (control.horizontal ? (control.availableHeight - height) / 2 : control.visualPosition * (control.availableHeight - height))
        implicitWidth: Tokens.controlHeight * 0.6
        implicitHeight: Tokens.controlHeight * 0.6
        radius: width / 2
        color: control.pressed ? Tokens.overAccent : Tokens.textStrong
        border.width: control.activeFocus ? Tokens.ringThickness : Tokens.hoverBorder
        border.color: control.activeFocus ? Tokens.accent : Tokens.border
    }
    background: Rectangle {
        x: control.leftPadding + (control.horizontal ? 0 : (control.availableWidth - width) / 2)
        y: control.topPadding + (control.horizontal ? (control.availableHeight - height) / 2 : 0)
        implicitWidth: control.horizontal ? Tokens.controlHeight * 5 : Tokens.base * 1.5
        implicitHeight: control.horizontal ? Tokens.base * 1.5 : Tokens.controlHeight * 5
        width: control.horizontal ? control.availableWidth : implicitWidth
        height: control.horizontal ? implicitHeight : control.availableHeight
        radius: height / 2
        color: Tokens.inputSurface
        scale: control.horizontal && control.mirrored ? -1 : 1
        Rectangle {
            y: control.horizontal ? 0 : control.visualPosition * parent.height
            width: control.horizontal ? control.position * parent.width : parent.width
            height: control.horizontal ? parent.height : control.position * parent.height
            radius: parent.radius
            color: Tokens.accent
        }
    }
}
