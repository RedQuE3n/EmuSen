import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A menu separator in the tokens: one rule across the row.
T.MenuSeparator {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    padding: Kit.Tokens.base
    verticalPadding: Kit.Tokens.base * 2
    contentItem: Rectangle {
        implicitWidth: Kit.Tokens.controlHeight * 5
        implicitHeight: Kit.Tokens.hoverBorder
        color: Kit.Tokens.rule
    }
}
