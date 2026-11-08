import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A tab bar in the tokens: its tabs in a row over a rule; the bar's focus flows to the current tab.
T.TabBar {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding)
    spacing: Kit.Tokens.base
    contentItem: ListView {
        focus: true
        model: control.contentModel
        currentIndex: control.currentIndex
        spacing: control.spacing
        orientation: ListView.Horizontal
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.AutoFlickIfNeeded
        snapMode: ListView.SnapToItem
        highlightMoveDuration: Kit.Tokens.instant
        highlightRangeMode: ListView.ApplyRange
        preferredHighlightBegin: Kit.Tokens.controlHeight
        preferredHighlightEnd: width - Kit.Tokens.controlHeight
    }
    background: Rectangle {
        color: Kit.Tokens.surface
        Rectangle {
            anchors.bottom: parent.bottom
            width: parent.width
            height: Kit.Tokens.hoverBorder
            color: Kit.Tokens.rule
        }
    }
}
