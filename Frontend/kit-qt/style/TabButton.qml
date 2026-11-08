import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A tab in the tokens: strong text when current, muted otherwise, the accent bar under the current tab, the ring on focus; the current tab holds the bar's focus so the pad walks the tabs.
T.TabButton {
    id: control
    Keys.onReturnPressed: control.clicked()
    Keys.onEnterPressed: control.clicked()
    focus: control.TabBar.tabBar ? control.TabBar.index === control.TabBar.tabBar.currentIndex : false
    implicitWidth: Math.ceil(Math.max(Kit.Tokens.controlHeight * 3, implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding))
    implicitHeight: Math.ceil(Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding))
    padding: Kit.Tokens.base * 2
    horizontalPadding: Kit.Tokens.base * 4
    spacing: Kit.Tokens.base * 2
    font.pixelSize: Kit.Tokens.fontBody
    icon.width: Kit.Tokens.fontBody * 1.5
    icon.height: Kit.Tokens.fontBody * 1.5
    contentItem: IconLabel {
        spacing: control.spacing
        mirrored: control.mirrored
        display: control.display
        icon: control.icon
        text: control.text
        font: control.font
        color: control.checked ? Kit.Tokens.textStrong : (control.enabled ? Kit.Tokens.muted : Kit.Tokens.border)
    }
    background: Rectangle {
        implicitHeight: Kit.Tokens.controlHeight
        radius: Kit.Tokens.rowRadius
        color: control.down || control.hovered ? Kit.Tokens.surfaceRaised : Kit.Tokens.surface
        border.width: control.activeFocus ? Kit.Tokens.ringThickness : 0
        border.color: Kit.Tokens.accent
        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: Kit.Tokens.barWidth
            color: Kit.Tokens.accent
            visible: control.checked
        }
    }
}
