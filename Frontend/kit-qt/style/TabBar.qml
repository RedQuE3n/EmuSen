import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A tab bar in the tokens: its tabs in a row over a rule.
T.TabBar {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding)
    spacing: Tokens.base
    contentItem: ListView {
        model: control.contentModel
        currentIndex: control.currentIndex
        spacing: control.spacing
        orientation: ListView.Horizontal
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.AutoFlickIfNeeded
        snapMode: ListView.SnapToItem
        highlightMoveDuration: Tokens.instant
        highlightRangeMode: ListView.ApplyRange
        preferredHighlightBegin: Tokens.controlHeight
        preferredHighlightEnd: width - Tokens.controlHeight
    }
    background: Rectangle {
        color: Tokens.surface
        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: Tokens.hoverBorder
            color: Tokens.rule
        }
    }
}
