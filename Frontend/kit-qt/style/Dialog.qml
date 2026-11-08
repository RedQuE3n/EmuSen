import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit as Kit

// A dialog in the tokens: a title in strong text over the raised surface, its buttons in a DialogButtonBox.
T.Dialog {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, contentWidth + leftPadding + rightPadding, implicitHeaderWidth, implicitFooterWidth)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding + (implicitHeaderHeight > 0 ? implicitHeaderHeight + spacing : 0) + (implicitFooterHeight > 0 ? implicitFooterHeight + spacing : 0))
    padding: Kit.Tokens.paneGutter
    spacing: Kit.Tokens.base * 2
    background: Rectangle {
        radius: Kit.Tokens.tileRadius
        color: Kit.Tokens.surfaceRaised
        border.width: Kit.Tokens.hoverBorder
        border.color: Kit.Tokens.border
    }
    header: Label {
        text: control.title
        visible: control.title
        elide: Label.ElideRight
        font.pixelSize: Kit.Tokens.fontTitle
        color: Kit.Tokens.textStrong
        padding: Kit.Tokens.paneGutter
        bottomPadding: 0
    }
    footer: DialogButtonBox {
        visible: count > 0
    }
    T.Overlay.modal: Rectangle {
        color: Kit.Tokens.scrim
    }
    T.Overlay.modeless: Rectangle {
        color: Kit.Tokens.scrim
        opacity: 0.25
    }
}
