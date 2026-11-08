import QtQuick
import QtQuick.Templates as T
import EmuSen.Kit

// A label in the tokens: body text in Text, links in the accent.
T.Label {
    id: control
    color: control.enabled ? Tokens.text : Tokens.muted
    linkColor: Tokens.accent
    font.pixelSize: Tokens.fontBody
}
