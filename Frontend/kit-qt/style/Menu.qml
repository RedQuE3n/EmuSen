import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A menu in the tokens: its rows on the raised surface with a border, the scrim behind a modal one.
T.Menu {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding)
    margins: 0
    overlap: Kit.Tokens.hoverBorder
    padding: Kit.Tokens.base
    delegate: MenuItem {}
    contentItem: ListView {
        implicitHeight: contentHeight
        model: control.contentModel
        interactive: Window.window ? contentHeight + control.topPadding + control.bottomPadding > control.height : false
        clip: true
        currentIndex: control.currentIndex
        highlightMoveDuration: Kit.Tokens.instant
        T.ScrollBar.vertical: ScrollBar {}
    }
    background: Rectangle {
        implicitWidth: Kit.Tokens.controlHeight * 5
        implicitHeight: Kit.Tokens.rowHeight
        radius: Kit.Tokens.rowRadius
        color: Kit.Tokens.surfaceRaised
        border.width: Kit.Tokens.hoverBorder
        border.color: Kit.Tokens.border
    }
    T.Overlay.modal: Rectangle {
        color: Kit.Tokens.scrim
    }
    T.Overlay.modeless: Rectangle {
        color: Kit.Tokens.scrim
        opacity: 0.25
    }
}
