import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A menu row in the tokens: a row's height, the focus bar at its left, a check mark drawn in the accent.
T.MenuItem {
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
        readonly property real arrowPadding: control.subMenu && control.arrow ? control.arrow.width + control.spacing : 0
        readonly property real indicatorPadding: control.checkable && control.indicator ? control.indicator.width + control.spacing : 0
        leftPadding: !control.mirrored ? indicatorPadding : arrowPadding
        rightPadding: control.mirrored ? indicatorPadding : arrowPadding
        spacing: control.spacing
        mirrored: control.mirrored
        display: control.display
        alignment: Qt.AlignLeft
        icon: control.icon
        text: control.text
        font: control.font
        color: control.highlighted ? Tokens.textStrong : (control.enabled ? Tokens.text : Tokens.muted)
    }
    indicator: Rectangle {
        x: control.mirrored ? control.width - width - control.rightPadding : control.leftPadding
        y: control.topPadding + (control.availableHeight - height) / 2
        visible: control.checkable
        implicitWidth: Tokens.fontRow
        implicitHeight: Tokens.fontRow
        radius: width / 2
        color: control.checked ? Tokens.accent : Tokens.inputSurface
        border.width: Tokens.hoverBorder
        border.color: Tokens.border
    }
    arrow: Rectangle {
        x: control.mirrored ? control.padding : control.width - width - control.padding
        y: control.topPadding + (control.availableHeight - height) / 2
        visible: control.subMenu
        implicitWidth: Tokens.base * 2
        implicitHeight: Tokens.base * 2
        rotation: 45
        color: Tokens.text
    }
    background: Rectangle {
        implicitHeight: Tokens.rowHeight
        radius: Tokens.rowRadius
        color: control.highlighted ? Tokens.surface : Tokens.surfaceRaised
        Rectangle {
            width: Tokens.barWidth
            height: parent.height
            radius: Tokens.rowRadius
            color: Tokens.accent
            visible: control.highlighted
        }
    }
}
