import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A menu separator in the tokens: one rule across the row.
T.MenuSeparator {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    padding: Tokens.base
    verticalPadding: Tokens.base * 2
    contentItem: Rectangle {
        implicitWidth: Tokens.controlHeight * 5
        implicitHeight: Tokens.hoverBorder
        color: Tokens.rule
    }
}
