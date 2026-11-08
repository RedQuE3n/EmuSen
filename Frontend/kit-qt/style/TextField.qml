import QtQuick
import QtQuick.Templates as T
import QtQuick.Controls.impl
import EmuSen.Kit

// A text field in the tokens: the input surface, a border that becomes the ring on focus, muted placeholder words.
T.TextField {
    id: control
    implicitWidth: implicitBackgroundWidth + leftInset + rightInset || Math.max(contentWidth, placeholder.implicitWidth) + leftPadding + rightPadding
    implicitHeight: Math.max(implicitBackgroundHeight + topInset + bottomInset, contentHeight + topPadding + bottomPadding, placeholder.implicitHeight + topPadding + bottomPadding)
    padding: Tokens.base * 2
    leftPadding: Tokens.base * 3
    font.pixelSize: Tokens.fontBody
    color: control.enabled ? Tokens.text : Tokens.muted
    selectionColor: Tokens.accent
    selectedTextColor: Tokens.overAccent
    placeholderTextColor: Tokens.muted
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
        implicitWidth: Tokens.controlHeight * 5
        implicitHeight: Tokens.controlHeight
        radius: Tokens.rowRadius
        color: Tokens.inputSurface
        border.width: control.activeFocus ? Tokens.ringThickness : Tokens.hoverBorder
        border.color: control.activeFocus ? Tokens.accent : Tokens.border
    }
}
