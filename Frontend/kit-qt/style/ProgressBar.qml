import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A progress bar in the tokens: an input-surface track, the accent to the value.
T.ProgressBar {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    contentItem: Item {
        implicitWidth: Tokens.controlHeight * 5
        implicitHeight: Tokens.base * 1.5
        Rectangle {
            width: control.indeterminate ? parent.width / 3 : control.visualPosition * parent.width
            height: parent.height
            radius: height / 2
            color: Tokens.accent
        }
    }
    background: Rectangle {
        implicitWidth: Tokens.controlHeight * 5
        implicitHeight: Tokens.base * 1.5
        radius: height / 2
        color: Tokens.inputSurface
    }
}
