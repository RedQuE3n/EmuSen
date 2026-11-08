pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A dialog's button row in the tokens: its buttons in a row on the raised surface.
T.DialogButtonBox {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding)
    spacing: Kit.Tokens.base * 2
    padding: Kit.Tokens.paneGutter
    topPadding: Kit.Tokens.base * 2
    alignment: Qt.AlignRight
    delegate: Button {
        width: control.count === 1 ? control.availableWidth / 2 : undefined
    }
    contentItem: ListView {
        implicitWidth: contentWidth
        implicitHeight: Kit.Tokens.controlHeight
        model: control.contentModel
        spacing: control.spacing
        orientation: ListView.Horizontal
        boundsBehavior: Flickable.StopAtBounds
        snapMode: ListView.SnapToItem
    }
    background: Rectangle {
        implicitHeight: Kit.Tokens.controlHeight
        radius: Kit.Tokens.tileRadius
        color: Kit.Tokens.surfaceRaised
    }
}
