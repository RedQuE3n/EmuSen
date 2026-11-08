import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A scroll bar in the tokens: a thin border-coloured handle, the accent while pressed, shown while active.
T.ScrollBar {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    padding: Tokens.base / 2
    visible: control.policy !== T.ScrollBar.AlwaysOff
    minimumSize: orientation === Qt.Horizontal ? height / width : width / height
    contentItem: Rectangle {
        implicitWidth: control.interactive ? Tokens.base * 1.5 : Tokens.base / 2
        implicitHeight: control.interactive ? Tokens.base * 1.5 : Tokens.base / 2
        radius: width / 2
        color: control.pressed ? Tokens.accent : Tokens.border
        opacity: control.policy === T.ScrollBar.AlwaysOn || (control.active && control.size < 1.0) ? 0.75 : 0
        Behavior on opacity {
            NumberAnimation { duration: Tokens.fade; easing.type: Tokens.fadeCurve }
        }
    }
}
