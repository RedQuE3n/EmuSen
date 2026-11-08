import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A list row in the tokens: a row's height, the focus bar at its left, the raised surface when focused or highlighted.
T.ItemDelegate {
    id: control
    Keys.onReturnPressed: control.clicked()
    Keys.onEnterPressed: control.clicked()
    implicitWidth: Math.ceil(Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding))
    implicitHeight: Math.ceil(Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding))
    padding: Kit.Tokens.base * 2
    leftPadding: Kit.Tokens.base * 4
    spacing: Kit.Tokens.base * 2
    font.pixelSize: Kit.Tokens.fontRow
    icon.width: Kit.Tokens.fontRow * 1.5
    icon.height: Kit.Tokens.fontRow * 1.5
    contentItem: IconLabel {
        spacing: control.spacing
        mirrored: control.mirrored
        display: control.display
        alignment: Qt.AlignLeft
        icon: control.icon
        text: control.text
        font: control.font
        color: control.highlighted ? Kit.Tokens.overAccent : (control.activeFocus ? Kit.Tokens.textStrong : (control.enabled ? Kit.Tokens.text : Kit.Tokens.muted))
    }
    background: Rectangle {
        implicitHeight: Kit.Tokens.rowHeight
        radius: Kit.Tokens.rowRadius
        color: control.highlighted ? Kit.Tokens.accent : (control.activeFocus || control.down ? Kit.Tokens.surfaceRaised : (control.hovered ? Kit.Tokens.inputSurface : Kit.Tokens.surface))
        Rectangle {
            width: Kit.Tokens.barWidth
            height: parent.height
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.accent
            visible: control.activeFocus && !control.highlighted
        }
    }
}
