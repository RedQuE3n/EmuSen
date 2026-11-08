import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A menu in the tokens: its rows on the raised surface with a border, the scrim behind a modal one.
T.Menu {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    margins: 0
    overlap: Tokens.hoverBorder
    padding: Tokens.base
    delegate: MenuItem {}
    contentItem: ListView {
        implicitHeight: contentHeight
        model: control.contentModel
        interactive: Window.window ? contentHeight + control.topPadding + control.bottomPadding > control.height : false
        clip: true
        currentIndex: control.currentIndex
        highlightMoveDuration: Tokens.instant
        T.ScrollBar.vertical: ScrollBar {}
    }
    background: Rectangle {
        implicitWidth: Tokens.controlHeight * 5
        implicitHeight: Tokens.rowHeight
        radius: Tokens.rowRadius
        color: Tokens.surfaceRaised
        border.width: Tokens.hoverBorder
        border.color: Tokens.border
    }
    T.Overlay.modal: Rectangle {
        color: Tokens.scrim
    }
    T.Overlay.modeless: Rectangle {
        color: Tokens.scrim
        opacity: 0.25
    }
}
