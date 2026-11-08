import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A menu row in the tokens: a row's height, the focus bar at its left, a check mark drawn in the accent.
T.MenuItem {
    id: control
    Keys.onReturnPressed: control.triggered()
    Keys.onEnterPressed: control.triggered()
    implicitWidth: Math.ceil(Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding))
    implicitHeight: Math.ceil(Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding))
    padding: Kit.Tokens.base * 2
    leftPadding: Kit.Tokens.base * 4
    spacing: Kit.Tokens.base * 2
    font.pixelSize: Kit.Tokens.fontRow
    icon.width: Kit.Tokens.fontRow * 1.5
    icon.height: Kit.Tokens.fontRow * 1.5
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
        color: control.highlighted ? Kit.Tokens.textStrong : (control.enabled ? Kit.Tokens.text : Kit.Tokens.muted)
    }
    indicator: Rectangle {
        x: control.mirrored ? control.width - width - control.rightPadding : control.leftPadding
        y: control.topPadding + (control.availableHeight - height) / 2
        visible: control.checkable
        implicitWidth: Kit.Tokens.fontRow
        implicitHeight: Kit.Tokens.fontRow
        radius: width / 2
        color: control.checked ? Kit.Tokens.accent : Kit.Tokens.inputSurface
        border.width: Kit.Tokens.hoverBorder
        border.color: Kit.Tokens.border
    }
    arrow: Rectangle {
        x: control.mirrored ? control.padding : control.width - width - control.padding
        y: control.topPadding + (control.availableHeight - height) / 2
        visible: control.subMenu
        implicitWidth: Kit.Tokens.base * 2
        implicitHeight: Kit.Tokens.base * 2
        rotation: 45
        color: Kit.Tokens.text
    }
    background: Rectangle {
        implicitHeight: Kit.Tokens.rowHeight
        radius: Kit.Tokens.rowRadius
        color: control.highlighted ? Kit.Tokens.surface : Kit.Tokens.surfaceRaised
        Rectangle {
            width: Kit.Tokens.barWidth
            height: parent.height
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.accent
            visible: control.highlighted
        }
    }
}
