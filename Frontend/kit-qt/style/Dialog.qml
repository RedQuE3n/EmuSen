import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A dialog in the tokens: a title in strong text over the raised surface, its buttons in a DialogButtonBox.
T.Dialog {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding, implicitHeaderWidth, implicitFooterWidth)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding + (implicitHeaderHeight > 0 ? implicitHeaderHeight + spacing : 0) + (implicitFooterHeight > 0 ? implicitFooterHeight + spacing : 0))
    padding: Tokens.paneGutter
    spacing: Tokens.base * 2
    background: Rectangle {
        radius: Tokens.tileRadius
        color: Tokens.surfaceRaised
        border.width: Tokens.hoverBorder
        border.color: Tokens.border
    }
    header: Label {
        text: control.title
        visible: control.title
        elide: Label.ElideRight
        font.pixelSize: Tokens.fontTitle
        color: Tokens.textStrong
        padding: Tokens.paneGutter
        bottomPadding: 0
    }
    footer: DialogButtonBox {
        visible: count > 0
    }
    T.Overlay.modal: Rectangle {
        color: Tokens.scrim
    }
    T.Overlay.modeless: Rectangle {
        color: Tokens.scrim
        opacity: 0.25
    }
}
