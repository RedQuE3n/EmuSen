import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A list row in the tokens: a row's height, the focus bar at its left, the raised surface when focused or highlighted.
T.ItemDelegate {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding)
    padding: Tokens.base * 2
    leftPadding: Tokens.base * 4
    spacing: Tokens.base * 2
    font.pixelSize: Tokens.fontRow
    icon.width: Tokens.fontRow * 1.5
    icon.height: Tokens.fontRow * 1.5
    contentItem: IconLabel {
        spacing: control.spacing
        mirrored: control.mirrored
        display: control.display
        alignment: Qt.AlignLeft
        icon: control.icon
        text: control.text
        font: control.font
        color: control.highlighted ? Tokens.overAccent : (control.activeFocus ? Tokens.textStrong : (control.enabled ? Tokens.text : Tokens.muted))
    }
    background: Rectangle {
        implicitHeight: Tokens.rowHeight
        radius: Tokens.rowRadius
        color: control.highlighted ? Tokens.accent : (control.activeFocus || control.down ? Tokens.surfaceRaised : (control.hovered ? Tokens.inputSurface : Tokens.surface))
        Rectangle {
            width: Tokens.barWidth
            height: parent.height
            radius: Tokens.rowRadius
            color: Tokens.accent
            visible: control.activeFocus && !control.highlighted
        }
    }
}
