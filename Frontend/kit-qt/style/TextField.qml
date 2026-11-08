import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit as Kit

// A text field in the tokens: the input surface, a border that becomes the ring on focus, muted placeholder words.
T.TextField {
    id: control
    implicitWidth: implicitBackgroundWidth + leftInset + rightInset || Math.max(contentWidth, placeholder.implicitWidth) + leftPadding + rightPadding
    implicitHeight: Math.ceil(Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding, placeholder.implicitHeight + topPadding + bottomPadding))
    padding: Kit.Tokens.base * 2
    leftPadding: Kit.Tokens.base * 3
    font.pixelSize: Kit.Tokens.fontBody
    color: control.enabled ? Kit.Tokens.text : Kit.Tokens.muted
    selectionColor: Kit.Tokens.accent
    selectedTextColor: Kit.Tokens.overAccent
    placeholderTextColor: Kit.Tokens.muted
    verticalAlignment: TextInput.AlignVCenter
    PlaceholderText {
        id: placeholder
        x: control.leftPadding
        y: control.topPadding
        width: control.width - (control.leftPadding + control.rightPadding)
        height: control.height - (control.topPadding + control.bottomPadding)
        text: control.placeholderText
        font: control.font
        color: control.placeholderTextColor
        verticalAlignment: control.verticalAlignment
        visible: !control.length && !control.preeditText && (!control.activeFocus || control.horizontalAlignment !== Qt.AlignHCenter)
        elide: Text.ElideRight
        renderType: control.renderType
    }
    background: Rectangle {
        implicitWidth: Kit.Tokens.controlHeight * 5
        implicitHeight: Kit.Tokens.controlHeight
        radius: Kit.Tokens.rowRadius
        color: Kit.Tokens.inputSurface
        border.width: control.activeFocus ? Kit.Tokens.ringThickness : Kit.Tokens.hoverBorder
        border.color: control.activeFocus ? Kit.Tokens.accent : Kit.Tokens.border
    }
}
