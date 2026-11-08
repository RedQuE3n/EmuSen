pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A dialog's button row in the tokens: its buttons in a row on the raised surface.
T.DialogButtonBox {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding)
    spacing: Tokens.base * 2
    padding: Tokens.paneGutter
    topPadding: Tokens.base * 2
    alignment: Qt.AlignRight
    delegate: Button {
        width: control.count === 1 ? control.availableWidth / 2 : undefined
    }
    contentItem: ListView {
        implicitWidth: contentWidth
        implicitHeight: Tokens.controlHeight
        model: control.contentModel
        spacing: control.spacing
        orientation: ListView.Horizontal
        boundsBehavior: Flickable.StopAtBounds
        snapMode: ListView.SnapToItem
    }
    background: Rectangle {
        implicitHeight: Tokens.controlHeight
        radius: Tokens.tileRadius
        color: Tokens.surfaceRaised
    }
}
