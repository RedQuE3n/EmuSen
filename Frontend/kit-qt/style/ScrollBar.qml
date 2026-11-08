import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A scroll bar in the tokens: a thin border-coloured handle, the accent while pressed, shown while active.
T.ScrollBar {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    padding: Kit.Tokens.base / 2
    visible: control.policy !== T.ScrollBar.AlwaysOff
    minimumSize: orientation === Qt.Horizontal ? height / width : width / height
    contentItem: Rectangle {
        implicitWidth: control.interactive ? Kit.Tokens.base * 1.5 : Kit.Tokens.base / 2
        implicitHeight: control.interactive ? Kit.Tokens.base * 1.5 : Kit.Tokens.base / 2
        radius: width / 2
        color: control.pressed ? Kit.Tokens.accent : Kit.Tokens.border
        opacity: control.policy === T.ScrollBar.AlwaysOn || (control.active && control.size < 1.0) ? 0.75 : 0
        Behavior on opacity {
            NumberAnimation { duration: Kit.Tokens.fade; easing.type: Kit.Tokens.fadeCurve }
        }
    }
}
