import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A progress bar in the tokens: an input-surface track, the accent to the value.
T.ProgressBar {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    contentItem: Item {
        implicitWidth: Kit.Tokens.controlHeight * 5
        implicitHeight: Kit.Tokens.base * 1.5
        Rectangle {
            width: control.indeterminate ? parent.width / 3 : control.visualPosition * parent.width
            height: parent.height
            radius: height / 2
            color: Kit.Tokens.accent
        }
    }
    background: Rectangle {
        implicitWidth: Kit.Tokens.controlHeight * 5
        implicitHeight: Kit.Tokens.base * 1.5
        radius: height / 2
        color: Kit.Tokens.inputSurface
    }
}
