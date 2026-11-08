pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A combo box in the tokens: an input-surface field, a drawn chevron, its list in a raised popup.
T.ComboBox {
    id: control
    implicitWidth: Math.max(implicitBackgroundWidth + leftInset + rightInset, implicitContentWidth + leftPadding + rightPadding)
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, implicitContentHeight + topPadding + bottomPadding, implicitIndicatorHeight + topPadding + bottomPadding)
    padding: Tokens.base * 2
    leftPadding: padding + (!control.mirrored || !indicator || !indicator.visible ? Tokens.base : indicator.width + spacing)
    rightPadding: padding + (control.mirrored || !indicator || !indicator.visible ? Tokens.base : indicator.width + spacing)
    spacing: Tokens.base * 2
    font.pixelSize: Tokens.fontBody
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
        y: control.topPadding + (control.availableHeight - height) / 2 - Tokens.base / 2
        implicitWidth: Tokens.base * 2.5
        implicitHeight: Tokens.base * 2.5
        rotation: 45
        color: Tokens.inputSurface
        border.width: Tokens.hoverBorder * 2
        border.color: control.enabled ? Tokens.text : Tokens.muted
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
        color: control.enabled ? Tokens.text : Tokens.muted
        selectionColor: Tokens.accent
        selectedTextColor: Tokens.overAccent
        verticalAlignment: Text.AlignVCenter
    }
    background: Rectangle {
        implicitWidth: Tokens.controlHeight * 4
        implicitHeight: Tokens.controlHeight
        radius: Tokens.rowRadius
        color: Tokens.inputSurface
        border.width: control.activeFocus ? Tokens.ringThickness : Tokens.hoverBorder
        border.color: control.activeFocus ? Tokens.accent : Tokens.border
    }
    popup: T.Popup {
        y: control.height
        width: control.width
        height: Math.min(contentItem.implicitHeight, Tokens.rowHeight * 8)
        topMargin: Tokens.base * 2
        bottomMargin: Tokens.base * 2
        padding: Tokens.base
        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: control.delegateModel
            currentIndex: control.highlightedIndex
            highlightMoveDuration: Tokens.instant
            T.ScrollBar.vertical: ScrollBar {}
        }
        background: Rectangle {
            radius: Tokens.rowRadius
            color: Tokens.surfaceRaised
            border.width: Tokens.hoverBorder
            border.color: Tokens.border
        }
    }
}
