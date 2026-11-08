pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A combo box in the tokens: an input-surface field, a drawn chevron, its list in a raised popup.
T.ComboBox {
    id: control
    implicitWidth: Math.ceil(Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding))
    implicitHeight: Math.ceil(Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding))
    padding: Kit.Tokens.base * 2
    leftPadding: padding + (!control.mirrored || !indicator || !indicator.visible ? Kit.Tokens.base : indicator.width + spacing)
    rightPadding: padding + (control.mirrored || !indicator || !indicator.visible ? Kit.Tokens.base : indicator.width + spacing)
    spacing: Kit.Tokens.base * 2
    font.pixelSize: Kit.Tokens.fontBody
    delegate: ItemDelegate {
        required property var model
        required property int index
        width: ListView.view.width
        text: model[control.textRole]
        highlighted: control.highlightedIndex === index
        hoverEnabled: control.hoverEnabled
    }
    indicator: Rectangle {
        x: control.mirrored ? control.padding : control.width - width - control.padding
        y: control.topPadding + (control.availableHeight - height) / 2 - Kit.Tokens.base / 2
        implicitWidth: Kit.Tokens.base * 2.5
        implicitHeight: Kit.Tokens.base * 2.5
        rotation: 45
        color: Kit.Tokens.inputSurface
        border.width: Kit.Tokens.hoverBorder * 2
        border.color: control.enabled ? Kit.Tokens.text : Kit.Tokens.muted
    }
    contentItem: T.TextField {
        implicitHeight: contentHeight + topPadding + bottomPadding
        topPadding: 0
        bottomPadding: 0
        text: control.editable ? control.editText : control.displayText
        enabled: control.editable
        autoScroll: control.editable
        readOnly: control.down
        inputMethodHints: control.inputMethodHints
        validator: control.validator
        selectByMouse: control.selectTextByMouse
        font: control.font
        color: control.enabled ? Kit.Tokens.text : Kit.Tokens.muted
        selectionColor: Kit.Tokens.accent
        selectedTextColor: Kit.Tokens.overAccent
        verticalAlignment: Text.AlignVCenter
    }
    background: Rectangle {
        implicitWidth: Kit.Tokens.controlHeight * 4
        implicitHeight: Kit.Tokens.controlHeight
        radius: Kit.Tokens.rowRadius
        color: Kit.Tokens.inputSurface
        border.width: control.activeFocus ? Kit.Tokens.ringThickness : Kit.Tokens.hoverBorder
        border.color: control.activeFocus ? Kit.Tokens.accent : Kit.Tokens.border
    }
    popup: T.Popup {
        y: control.height
        width: control.width
        height: Math.min(contentItem.implicitHeight, Kit.Tokens.rowHeight * 8)
        topMargin: Kit.Tokens.base * 2
        bottomMargin: Kit.Tokens.base * 2
        padding: Kit.Tokens.base
        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: control.delegateModel
            currentIndex: control.highlightedIndex
            highlightMoveDuration: Kit.Tokens.instant
            highlightResizeDuration: Kit.Tokens.instant
            T.ScrollBar.vertical: ScrollBar {}
        }
        background: Rectangle {
            radius: Kit.Tokens.rowRadius
            color: Kit.Tokens.surfaceRaised
            border.width: Kit.Tokens.hoverBorder
            border.color: Kit.Tokens.border
        }
    }
}
